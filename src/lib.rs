#![feature(braced_empty_structs)]
extern crate html5ever;
extern crate hyper;
extern crate libc;
extern crate string_cache;
extern crate tendril;
extern crate url;

use libc::c_char;

use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;
use std::ffi::CStr;
use std::io::Read;
use std::result::Result;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::{current, park, Thread};

use tendril::StrTendril;
use string_cache::Atom;

use hyper::Client;
use hyper::header::Connection;

use html5ever::parse_document;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::driver::{ParseOpts, BytesOpts};
use html5ever::rcdom::{Document, Doctype, Comment, Element, RcDom, Handle, Text};

use url::Url;

pub enum ProcessOutputs {
    Processes(Box<AtomicProcess>),
    Output(i64),
}

trait AtomicProcess {
    fn process_this(&self) -> Vec<ProcessOutputs>;
}

struct TaskQueue {
    threads: Arc<Mutex<Vec<Thread>>>,
    queue: Arc<Mutex<VecDeque<Box<AtomicProcess>>>>,
    output_channel: Sender<i64>,
}

struct PageDownloader {
    thread_url: String,
}

struct WalkDom {
    handle: Handle,
    count: i64,
}

struct PauseThread;
// The plan here is to have a set of tasks sitting of a FIFO queue
// each thread gets 1 task and processes it. Each new task that is created
// is added to the queue.

impl AtomicProcess for PauseThread {
    fn process_this(&self) -> Vec<ProcessOutputs> {
        park();
        vec![ProcessOutputs::Output(0i64)]
    }
}

impl TaskQueue {
    pub fn nextTask(&self) -> Box<AtomicProcess> {
        match self.queue.clone().lock().unwrap().pop_back() {
            Some(res) => res,
            None => Box::new(PauseThread),
        }
    }

    pub fn addTask(&self, new_task: Box<AtomicProcess>) {
        self.queue.clone().lock().unwrap().push_front(new_task);
        self.threads.clone().lock().unwrap().iter().map(|t| t.unpark());
    }

    pub fn addThreadToWorkers(&self, t_handle: Thread) {
        self.threads.clone().lock().unwrap().push(t_handle);
        loop {
            for process in self.nextTask().process_this() {
                match process {
                    ProcessOutputs::Processes(res) => self.addTask(res),
                    ProcessOutputs::Output(res) => self.output_channel.send(res).unwrap(),
                }
            }
        }
    }
}

impl AtomicProcess for WalkDom {
    fn process_this(&self) -> Vec<ProcessOutputs> {
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

        let dom_steps: Vec<ProcessOutputs> = node.children
                                                 .iter()
                                                 .map(|child| {
                                                     ProcessOutputs::Processes(Box::new(WalkDom {
                                                         handle: *child,
                                                         count: self.count,
                                                     }))
                                                 })
                                                 .collect();
        if dom_steps.len() == 0 {
            dom_steps.push(ProcessOutputs::Output(self.count));
        }
        dom_steps
    }
}

impl AtomicProcess for PageDownloader {
    fn process_this(&self) -> Vec<ProcessOutputs> {
        let client = Client::new();
        let res = client.get(&self.thread_url[..])
                        .header(Connection::close())
                        .send()
                        .unwrap();

        // Read the Response.
        let mut body = String::new();

        let mut input = StrTendril::new();
        let _ = input.try_push_bytes(body.as_bytes());
        let mut dom = parse_document(RcDom::default(), Default::default())
                          .from_utf8()
                          .process(input);


        vec![ProcessOutputs::Processes(Box::new(WalkDom {
                 handle: dom.document,
                 count: 0i64,
             }))]
    }
}

fn start_read_thread(url: String) {
    let (sender, receiver) = channel::<i64>();
    let queue_controller = TaskQueue {
        threads: Arc::new(Mutex::new(vec![])),
        queue: Arc::new(Mutex::new(VecDeque::new())),
        output_channel: sender,
    };
    (0..4)
        .map(|_| {
            let worker_thread = thread::spawn(move || {});
            queue_controller.addThreadToWorkers(*worker_thread.thread());
        })
        .collect();
    queue_controller.addTask(Box::new(PageDownloader { thread_url: url }));

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
