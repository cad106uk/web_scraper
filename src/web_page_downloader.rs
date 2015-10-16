/*
The function store_raw_html_page is usually the only function needed
I have however left the download_page function public just encase
*/
extern crate hyper;

use std::io::Read;
use std::sync::mpsc::Sender;

use hyper::Client;
use hyper::header::Connection;

pub fn download_page(url: String) -> String {
    let client = Client::new();
    let mut res = client.get(url.as_str()).header(Connection::close()).send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

pub fn store_raw_html_page(pages: Sender<String>, thread_url: String) {
    pages.send(download_page(thread_url)).unwrap()
}
