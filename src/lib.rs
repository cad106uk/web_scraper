#![feature(convert)]
#![feature(cstr_to_str)]
extern crate libc;
extern crate hyper;
extern crate html5ever;
extern crate tendril;
extern crate string_cache;

use std::result::Result;
use std::thread;
use std::string::String;
use std::io::Read;
use std::ffi::CStr;
use std::default::Default;
use libc::c_char;

use hyper::Client;
use hyper::header::Connection;

use html5ever::{parse, one_input};
use html5ever::rcdom::{Document, Doctype, Comment, Element, RcDom, Handle, Text};

use tendril::StrTendril;

use string_cache::atom::Atom;


fn worker(url: String) -> String {
    let client = Client::new();
    let mut res = client.get(url.as_str())
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

fn walk(indent: usize, handle: Handle, count: &mut Box<u64>) -> Result<u64, u64> {
    let node = handle.borrow();

    match node.node {
        Element(ref name, _, _) => {
            if name.local == Atom::from_slice("article") {
                **count += 1
            }
        },

        Document => (),

        Doctype(_, _, _) => (),

        Text(_) => (),

        Comment(_) => ()
    };


    for child in node.children.iter() {
        let res = walk(indent+4, child.clone(), count);

        match res {
            Ok(v) => println!("walk success with ={:?}", v),

            Err(e) => println!("walk errored with ={:?}", e)
        }
    }
    Ok(**count)
}

fn start_read_thread(url: String) {
    let thread_url = url.clone().to_string();
    let thr = thread::spawn(|| {
        let raw_html_page = worker(thread_url);
        let mut input = StrTendril::new();
        let _ = input.try_push_bytes(raw_html_page.as_bytes());

        let dom: RcDom = parse(one_input(input), Default::default());
        let mut output = Box::new(0u64);
        let res = walk(0, dom.document, &mut output);
        println!("output count {}", output);
        res
    });

    let res = thr.join();
    match res {
        Ok(v) => println!("Thread finished with count={:?}", v),
        Err(e) => println!("Thread errored with count={:?}", e),
    };
}

#[no_mangle]
pub extern "C" fn process(url: *const c_char) {
    let c_value = Some(unsafe {
        CStr::from_ptr(url).to_string_lossy().into_owned()
    });

    match c_value {
        Some(value) => start_read_thread(String::from(value.as_str())),
        None => ()
    }
}


#[test]
fn it_works() {
    let raw_html_page = worker("http://slashdot.org".to_string());
    let mut input = StrTendril::new();
    let _ = input.try_push_bytes(raw_html_page.as_bytes());

    let dom: RcDom = parse(one_input(input), Default::default());
    let mut output = Box::new(0u64);
    let res = walk(0, dom.document, &mut output);
    assert_eq!(true, true);
    assert!(res.is_ok());
    assert_eq!(match res {
        Ok(val) => val,
        Err(e) => e
    }, 25);
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


// fn run_in_this_thread(url: String) {
//     let raw_html_page = worker(url.to_string());
//     let mut input = StrTendril::new();
//     let _ = input.try_push_bytes(raw_html_page.as_bytes());

//     let dom: RcDom = parse(one_input(input), Default::default());
//     let mut output = Box::new(0u64);
//     let res = walk(0, dom.document, &mut output);
//     match res {
//         Ok(v) => println!("Thread finished with count={:?} output={:?}", v, output),
//         Err(e) => println!("Thread errored with count={:?} output={:?}", e, output),
//     }
//     println!("Done");
// }
