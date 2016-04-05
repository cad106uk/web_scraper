// The plan here is to have a set of tasks sitting of a FIFO queue
// each thread gets 1 task and processes it. Each new task that is created
// is added to the queue.

#![feature(braced_empty_structs)]

use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{current, park, Thread};
use std::vec::Vec;

pub enum ProcessOutputs {
    Processes(Box<AtomicProcess>),
    Output(i64),
}

trait AtomicProcess {
    fn process_this(&self) -> ProcessOutputs;
}

struct PauseThread;
impl AtomicProcess for PauseThread {
    fn process_this(&self) -> ProcessOutputs {
        park();
        ProcessOutputs::Output(0i64)
    }
}

struct TaskQueue {
    threads: Arc<Mutex<Vec<Thread>>>,
    queue: Arc<Mutex<VecDeque<Box<AtomicProcess>>>>,
    output_channel: Sender<i64>,
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
            match self.nextTask().process_this() {
                ProcessOutputs::Processes(res) => self.addTask(res),
                ProcessOutputs::Output(res) => self.output_channel.send(res).unwrap(),
            }
        }
    }
}
