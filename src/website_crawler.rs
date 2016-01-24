// Given an URL, get the page for that URL
// Given a page of a website, get all the anchors that point to pages within that website
// Make sure that each intrnal anchor is only recorded once.
// visit ever url in the website record all the and pages and urls

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver};

use hyper::Client;
use hyper::header::Connection;

use html5ever::{parse, one_input};
use html5ever::rcdom::Element;

use url::Url;

let mut KnownWebsitePages = HashMap::new();

fn crawl_page(page_url: String) {
    let parsed_url = Url::parse(page_url).unwrap();

    let mut pages = KnownWebsitePagess.entry(
        parsed_url.domain().unwrap()
    ).or_insert(mut HashSet::new());

    if pages.insert(parsed_url) {
    }
}

fn handle_node_anchor(node: Element) {
    //Your moma
}

fn get_internal_links(page: RcDom) {
    let node = handle.borrow();

    match node.node {
        Element(ref nmae, _ _) => {
            if name.local == Atom::from("a") {
            }
        },

        _ => ()
    },
}

fn store_raw_html_page(pages: Sender<String>, thread_url: String) {
    pages.send(download_page(thread_url)).unwrap()
}

fn download_page(url: String) -> RcDom {
    let client = Client::new();
    let res = client.get(&url[..]).header(Connection::close()).send().unwrap();

    let mut body = String::new();
    let mut input = StrTendril::new();
    res.read_to_string(&mut body).unwrap();
    input.try_push_bytes(body.as_bytes());

    let parsed_dom: RcDom = parse(one_input(input));
    get_internal_links(parsed_dom);
    parsed_dom
}
