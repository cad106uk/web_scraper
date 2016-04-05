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
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use libc::c_char;

use html5ever::{parse, one_input};

use tendril::StrTendril;

use string_cache::Atom;

use url::Url;
use task_queue::{TaskQueue, AtomicProcess};

mod website_crawler;
mod task_queue;

fn start_read_thread(url: String) {
    let (sender, receiver) = channel::<i64>();
    let queue_controller = TaskQueue {
        threads: Arc::new(Mutex::new(vec![])),
        queue:
            Arc::new(Mutex::new(VecDeque::new(<Box<AtomicProcess>>::new()))),
        output_channel: sender,
    };
    (0..4).map(|| {
        let worker_thread = thread::spawn(|i| i);
        queue_controller.addThreadToWorkers(worker_thread.thread());
    });
    queue_controller.addTask(Box::new(website_crawler::PageDownloader { thread_url: url }));

    loop {
        println!("Thread finished with count={:?}", receiver.recv().unwrap());
    }
}

#[no_mangle]
pub extern "C" fn process(url: *const c_char) {
    let c_value = Some(unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() });

    match c_value {
        Some(value) => start_read_thread(String::from(&value[..])),
        None => (),
    }
}


// #[test]
// fn it_works() {
//     let raw_html_page = download_page("http://slashdot.org".to_string());
//     let mut input = StrTendril::new();
//     let _ = input.try_push_bytes(raw_html_page.as_bytes());

//     let dom: RcDom = parse(one_input(input), Default::default());
//     let mut output = Box::new(0u64);
//     let res = walk(dom.document, &mut output);
//     assert_eq!(*output, 26);
//     assert!(res.is_ok());
//     assert_eq!(match res {
//                    Ok(val) => val,
//                    Err(e) => e,
//                },
//                26);
// }


// fn walk(handle: Handle, count: &mut Box<u64>) -> Result<u64, u64> {
//     let node = handle.borrow();

//     match node.node {
//         Element(ref name, _, _) => {
//             if name.local == Atom::from("article") {
//                 **count += 1
//             }
//         }

// Document => (),

// Doctype(_, _, _) => (),

// Text(_) => (),

//         Comment(_) => (),
//     };


//     for child in node.children.iter() {
//         let res = walk(child.clone(), count);

//         match res {
//             Ok(v) => println!("walk success with ={:?}", v),

//             Err(e) => println!("walk errored with ={:?}", e),
//         }
//     }
//     Ok(**count)
// }
// fn process_next_page(raw_pages: Receiver<String>) -> Result<u64, u64> {
//     let raw_html_page = raw_pages.recv().unwrap();

//     let mut output = Box::new(0u64);
//     let mut input = StrTendril::new();
//     let _ = input.try_push_bytes(raw_html_page.as_bytes());
//     let page: RcDom = parse(one_input(input), Default::default());

//     let res = walk(page.document, &mut output);
//     println!("Output {:?}", output);
//     res
// }
