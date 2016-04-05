use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver};
use std::sync::mpsc::Sender;
use std::io::Read;
use std::default::Default;

use tendril::StrTendril;
use string_cache::Atom;

use hyper::Client;
use hyper::header::Connection;

use html5ever::{parse, one_input};
use html5ever::rcdom::{Document, Doctype, Comment, Element, RcDom, Handle, Text};

use url::Url;

use task_queue::{AtomicProcess, ProcessOutputs};

struct WalkDom {
    add_task: Sender<Box<AtomicProcess>>,
    handle: Handle,
    count: i64,
}

impl AtomicProcess for WalkDom {
    fn process_this(&self) -> ProcessOutputs {
        let node = self.handle.borrow();

        match node.node {
            Element(ref name, _, _) => {
                if name.local == Atom::from("article") {
                    self.count += 1
                }
            }

            Document => (),

            Doctype(_, _, _) => (),

            Text(_) => (),

            Comment(_) => (),
        };


        let child_iter = node.children.iter();
        match child_iter.next() {
            Some(child) => {
                let mut output = vec![Box::new(WalkDom {
                                          handle: *child,
                                          count: self.count,
                                      })];
                for c in child_iter {
                    output.push(Box::new(WalkDom {
                        handle: *c,
                        count: self.count,
                    }));
                }
                ProcessOutputs::Processes(output)
            }
            None => ProcessOutputs::Output(self.count),
        }
    }
}

struct ParsePage {
    raw_html_page: String,
}

impl AtomicProcess for ParsePage {
    fn process_this(&self) -> ProcessOutputs {
        let mut input = StrTendril::new();
        input.try_push_bytes(self.raw_html_page.as_bytes());

        ProcessOutputs::Processes(vec![Box::new(WalkDom {
                                           handle: parse(one_input(input), Default::default())
                                                       .document,
                                           count: 0i64,
                                       })])
    }
}

struct PageDownloader {
    thread_url: String,
}

impl AtomicProcess for PageDownloader {
    fn process_this(&self) -> ProcessOutputs {
        let client = Client::new();
        let res = client.get(&self.thread_url[..])
                        .header(Connection::close())
                        .send()
                        .unwrap();

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
        ProcessOutputs::Processes(vec![Box::new(ParsePage { raw_html_page: body })])
    }
}

// fn store_raw_html_page(pages: Sender<String>, thread_url: String) {
//     pages.send(download_page(thread_url)).unwrap()
// }

// fn download_page(url: String) -> String {
//     let client = Client::new();
//     let res = client.get(&url[..]).header(
//         Connection::close()
//     ).send().unwrap();

//     let mut body = String::new();
//     res.read_to_string(&mut body).unwrap();
// }

// fn parse_page(body: &String) -> RcDom {
//     let mut input = StrTendril::new();
//     input.try_push_bytes(body.as_bytes());

//     parse(one_input(input), Default::default())
// }

// fn crawl_site_from_page(parsed_dom: &RcDom) {
//     get_internal_links(*parsed_dom);
//     // body
//     // parsed_dom
// }
