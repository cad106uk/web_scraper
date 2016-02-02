extern crate libc;
extern crate html5ever;
extern crate tendril;
extern crate string_cache;
extern crate hyper;
extern crate url;

use std::collections::{HashMap, HashSet, VecDeque};
use std::result::Result;
use std::thread;
use std::string::String;
use std::ffi::CStr;
use std::default::Default;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use libc::c_char;

use html5ever::{parse, one_input};
use html5ever::rcdom::{Document, Doctype, Comment, Element, RcDom, Handle, Text};

use tendril::StrTendril;

use string_cache::Atom;

use url::Url;

use website_crawler::{download_page, store_raw_html_page};

mod website_crawler;


struct ThreadCoordinator {
    KnownWebsitePages: HashMap<String, HashSet<String>>,
    PagesToDownload: VecDeque<Url>,
    PagesToProcess: VecDeque<&String>,
    ParsedPagesForInternalAchors: VecDeque<&RcDom>,
    PagesToParse: VecDeque<&RcDom>,
}

impl ThreadCoordinator {
    fn get_page_if_needed(&self, url: String) {
        let page = match Url::parse(&url) {
            Ok(is_page) => is_page,
            Err(_) => return,
        };

        let mut visited_pages = match page.domain() {
            Some(domain) => {
                match self.KnownWebsitePages.get(domain) {
                    Some(pages) => pages,
                    None => return,
                }
            },
            None => return,
        };

        let path = match page.path() {
            Some(path_vec) => path_vec.join("/"),
            None => return,
        };
        if visited_pages.insert(path) {
            self.PagesToDownload.push_front(page);
        }
    }
}

fn start_read_thread(url: String) {
    // have multiple threads do this.
    // record url
    // download page
    // parse page
    // get all internal urls
    // for each new url goto record url step
    let mut tmp = Arc::new(Box::new(ThreadCoordinator {
        KnownWebsitePages: HashMap::new(),
        PagesToDownload: VecDeque::new(),
    }));
    let domain = String::from(Url::parse(&url).unwrap().domain().unwrap());

    // KnownWebsitePages.insert(domain, HashSet::new());

    // match KnownWebsitePages.get(&domain) {
    //     Some(pages) => {
    //         pages.insert("".to_string());
    //         pages.insert("/".to_string());
    //     },
    //     _ => ()
    // }
    let (page_to_download, download_page) = channel();

    let (page_to_parse, parse_page) = channel();
    let (sender, receiver) = channel::<String>();
    let thread_url = url.clone().to_string();

    let process_page = thread::spawn(|| process_next_page(receiver));
    thread::spawn(|| store_raw_html_page(sender, thread_url));

    let res = process_page.join();
    match res {
        Ok(v) => println!("Thread finished with count={:?}", v),
        Err(e) => println!("Thread errored with count={:?}", e),
    };
}

#[no_mangle]
pub extern "C" fn process(url: *const c_char) {
    let c_value = Some(unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() });

    match c_value {
        Some(value) => start_read_thread(String::from(&value[..])),
        None => (),
    }
}


#[test]
fn it_works() {
    let raw_html_page = download_page("http://slashdot.org".to_string());
    let mut input = StrTendril::new();
    let _ = input.try_push_bytes(raw_html_page.as_bytes());

    let dom: RcDom = parse(one_input(input), Default::default());
    let mut output = Box::new(0u64);
    let res = walk(dom.document, &mut output);
    assert_eq!(*output, 26);
    assert!(res.is_ok());
    assert_eq!(match res {
                   Ok(val) => val,
                   Err(e) => e,
               },
               26);
}


fn walk(handle: Handle, count: &mut Box<u64>) -> Result<u64, u64> {
    let node = handle.borrow();

    match node.node {
        Element(ref name, _, _) => {
            if name.local == Atom::from("article") {
                **count += 1
            }
        }

        Document => (),

        Doctype(_, _, _) => (),

        Text(_) => (),

        Comment(_) => (),
    };


    for child in node.children.iter() {
        let res = walk(child.clone(), count);

        match res {
            Ok(v) => println!("walk success with ={:?}", v),

            Err(e) => println!("walk errored with ={:?}", e),
        }
    }
    Ok(**count)
}

fn process_next_page(raw_pages: Receiver<String>) -> Result<u64, u64> {
    let raw_html_page = raw_pages.recv().unwrap();

    let mut output = Box::new(0u64);
    let mut input = StrTendril::new();
    let _ = input.try_push_bytes(raw_html_page.as_bytes());
    let page: RcDom = parse(one_input(input), Default::default());

    let res = walk(page.document, &mut output);
    println!("Output {:?}", output);
    res
}
