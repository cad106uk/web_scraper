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
use html5ever::rcdom::{Element, RcDom};

use url::Url;

use task_queue::make_task;


// fn crawl_page(page_url: String) {
//     let parsed_url = String::from(Url::parse(&page_url).unwrap().domain().unwrap());

//     unsafe {
//         let mut pages = KnownWebsitePages.entry(parsed_url)
//                                          .or_insert(HashSet::new());

//         if pages.insert(parsed_url) {
//         }
//     }
// }

// fn handle_node_anchor(node: Element) {
//     //Your moma
// }

fn get_internal_links(page: RcDom) {
    let node = page.document.borrow();

    match node.node {
        Element(ref name, _, _) => {
            if name.local == Atom::from("a") {
            }
        }

        _ => (),
    }
}



struct PageDownloader {
    thread_url: String
}

impl AtomicProcess for PageDownloader {
    fn process_this(&self) {
        let client = Client::new();
        let res = client.get(
            &self.thread_url[..]
        ).header(
            Connection::close()
        ).send().unwrap();

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
    }
}

fn store_raw_html_page(pages: Sender<String>, thread_url: String) {
    pages.send(download_page(thread_url)).unwrap()
}

fn download_page(url: String) -> String {
    let client = Client::new();
    let res = client.get(&url[..]).header(Connection::close()).send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    make_task!(parse_page, body)
}

fn parse_page(body: &String) -> RcDom {
    let mut input = StrTendril::new();
    input.try_push_bytes(body.as_bytes());

    parse(one_input(input), Default::default())
}

fn crawl_site_from_page(parsed_dom: &RcDom) {
    get_internal_links(*parsed_dom);
    // body
    // parsed_dom
}
