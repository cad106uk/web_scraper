#![feature(convert)]
#![feature(cstr_to_str)]
extern crate libc;
extern crate hyper;
extern crate html5ever;
extern crate tendril;

use std::result::Result;
use std::thread;
use std::string::String;
use std::io::Read;
use std::iter::repeat;
use std::ffi::CStr;
use std::default::Default;
use libc::c_char;

use hyper::Client;
use hyper::header::Connection;

use html5ever::{parse, one_input};
use html5ever::rcdom::{Document, Doctype, Comment, Element, RcDom, Handle, Text};

use tendril::StrTendril;

fn worker(url: String) -> String {
    let client = Client::new();
    let mut res = client.get(url.as_str())
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

fn walk(indent: usize, handle: Handle) -> Result<usize, usize> {
    let node = handle.borrow();
    // FIXME: don't allocate
    print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.node {
        Document
            => println!("#Document"),

        Doctype(ref name, ref public, ref system)
            => println!("#Doctype <!DOCTYPE {} \"{}\" \"{}\">", *name, *public, *system),

        Text(_)
            => println!("#text: Your Moma"),

        Comment(ref text)
            => println!("#Comment <!-- {} -->", text),

        Element(ref name, _, ref attrs) => {
            print!("#Element <{}", name.local);
            for attr in attrs.iter() {
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        }
    }

    let mut tmp = indent;
    for child in node.children.iter() {
        let res = walk(indent+4, child.clone());
        tmp = match res {
            Ok(v) => v,
            Err(e) => {
                println!("walk errored with ={:?}", e);
                e
            }
        }
    }
    Ok(tmp)
}

// fn start_read_thread(url: String) {
//     let thread_url = url.clone().to_string();
//     let handles: Vec<_> = (0..2).map(move |_| {
//         thread::spawn(|| {
//             let raw_html_page = worker(thread_url);
//             let mut input = StrTendril::new();
//             let _ = input.try_push_bytes(raw_html_page.as_bytes());

//             let dom: RcDom = parse(one_input(input), Default::default());
//             walk(0, dom.document);
//         })}).collect();

//     for h in handles {
//         let res = h.join();
//         match res {
//             Ok(v) => println!("Thread finished with count={:?}", v),
//             Err(e) => println!("Thread errored with count={:?}", e),
//         }

//     }
//     println!("Done");
// }

fn start_read_thread(url: String) {
    let thread_url = url.clone().to_string();
    let thr = thread::spawn(|| {
        let raw_html_page = worker(thread_url);
        let mut input = StrTendril::new();
        let _ = input.try_push_bytes(raw_html_page.as_bytes());

        let dom: RcDom = parse(one_input(input), Default::default());
        walk(0, dom.document);
    });

    let res = thr.join();
    match res {
        Ok(v) => println!("Thread finished with count={:?}", v),
        Err(e) => println!("Thread errored with count={:?}", e),
    }
}

fn run_in_this_thread(url: String) {
    let raw_html_page = worker(url.to_string());
    let mut input = StrTendril::new();
    let _ = input.try_push_bytes(raw_html_page.as_bytes());

    let dom: RcDom = parse(one_input(input), Default::default());
    let res = walk(0, dom.document);
    match res {
        Ok(v) => println!("Thread finished with count={:?}", v),
        Err(e) => println!("Thread errored with count={:?}", e),
    }
    println!("Done");
}

#[no_mangle]
pub extern "C" fn process(url: *const c_char) {
    let c_value = Some(unsafe {
        CStr::from_ptr(url).to_string_lossy().into_owned()
    });

    match c_value {
        Some(value) => start_read_thread(String::from(value.as_str())),
        None => {}
    }
}


#[test]
fn it_works() {
    let raw_html_page = worker("http://www.google.com".to_string());
    let mut input = StrTendril::new();
    let _ = input.try_push_bytes(raw_html_page.as_bytes());

    let dom: RcDom = parse(one_input(input), Default::default());
    let res = walk(0, dom.document);
    assert_eq!(true, true);
    assert!(res.is_ok());
    assert_eq!(match res {
        Ok(val) => val,
        Err(e) => e
    }, 20);
}
