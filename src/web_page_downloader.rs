/*
The function store_raw_html_page is usually the only function needed
I have however left the download_page function public just encase
*/
use std::io::Read;
use std::sync::mpsc::Sender;

pub fn store_raw_html_page(pages: Sender<String>, thread_url: String) {
    pages.send(download_page(thread_url)).unwrap()
}
