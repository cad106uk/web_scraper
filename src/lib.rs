extern crate hyper;
extern crate libc;

use std::thread;
use std::io::Read;
use std::ffi::CStr;
use libc::c_char;

use hyper::Client;
use hyper::header::Connection;

fn worker(url: &str) -> String {
    let client = Client::new();
    let mut res = client.get(url)
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

fn start_read_thread(url: &str) {
    let handles: Vec<_> = (0..2).map(
        |_| {thread::spawn(|| worker(url))
    }).collect();

    for h in handles {
        let res = h.join().map_err(|_| "Could not join a thread!");
        println!("Thread finished with count={}", res.unwrap());
    }
    println!("Done");
}

#[no_mangle]
pub extern "C" fn process(url: *const c_char) {
    let c_value = Some(unsafe {
        CStr::from_ptr(url).to_string_lossy().into_owned()
    });

    match c_value {
        Some(value) => start_read_thread(String::from_str(value.as_str())),
        None => {}
    }
}


#[test]
fn it_works() {
    assert_eq!(worker("http://www.icanhazip.com"), "85.133.27.34\n");
}
