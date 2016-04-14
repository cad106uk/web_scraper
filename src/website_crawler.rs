use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver};
use std::sync::mpsc::Sender;
use std::io::Read;
use std::default::Default;

use tendril::StrTendril;
use string_cache::Atom;

use hyper::Client;
use hyper::header::Connection;

use html5ever::parse_document;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::driver::{ParseOpts, BytesOpts};
use html5ever::rcdom::{Document, Doctype, Comment, Element, RcDom, Handle, Text};

use url::Url;

use task_queue::{AtomicProcess, ProcessOutputs};

struct WalkDom {
    handle: Handle,
    count: i64,
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

struct PageDownloader {
    thread_url: String,
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
