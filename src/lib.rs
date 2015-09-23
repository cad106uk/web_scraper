#![feature(convert)]
#![feature(cstr_to_str)]
extern crate libc;
extern crate hyper;
extern crate html5ever;
extern crate tendril;

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

fn walk(indent: usize, handle: Handle) {
    let node = handle.borrow();
    // FIXME: don't allocate
    print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.node {
        Document
            => println!("#Document"),

        Doctype(ref name, ref public, ref system)
            => println!("<!DOCTYPE {} \"{}\" \"{}\">", *name, *public, *system),

        Text(ref text)
            => println!("#text: {}", text),

        Comment(ref text)
            => println!("<!-- {} -->", text),

        Element(ref name, _, ref attrs) => {
            print!("<{}", name.local);
            for attr in attrs.iter() {
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        }
    }

    for child in node.children.iter() {
        walk(indent+4, child.clone());
    }
}

fn start_read_thread(url: String) {
    let handles: Vec<_> = (0..2).map(|_| {
        let thread_url = url.clone();
        thread::spawn(|| {
            worker(thread_url)
        })}).collect();

    for h in handles {
        let res = h.join(); //.map_err(|val| val);
        match res {
            Ok(v) => println!("Thread finished with count={}", v),
            Err(e) => println!("Thread errored with count={:?}", e),
        }

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
    walk(0, dom.document);
    assert_eq!(true, true);
}
