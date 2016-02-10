/*
The plan here is to have a set of tasks sitting of a FIFO queue
each thread gets 1 task and processes it. Each new task that is created
is added to the queue.
 */

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{current, park, Thread};
use std::vec::Vec;

trait AtomicProcess {
    fn process_this(&self) -> Vec<Box<AtomicProcess>>;
}

macro_rules! make_task {
    ($($head:expr; $tail:expr) => {{
        struct task_from_macro {
        }

        impl AtomicProcess for task_from_macro {
            fn process_this(&self) -> Vec<Box<AtomicProcess>> {
                $head($($tail),*)
            }
        }
    }};
}

struct TaskQueue {
    threads: Arc<Mutex<Vec<Thread>>>,
    queue: Arc<Mutex<VecDeque<Box<AtomicProcess>>>>,
}

impl TaskQueue {
    pub fn nextTask(&self) -> Option<AtomicProcess> {
        match self.queue.clone().lock().unwrap().pop_back() {
            Some(res) => &res,
            None => park(current()),
        }
    }

    pub fn addTask(&self, new_task: AtomicProcess) {
        self.queue.clone().lock().unwrap().push_front(Box::new(new_task));
        for t in self.threads.iter() {
            t.unpark();
        }
    }
}

fn start_task_queue_thread(queue: TaskQueue) {
    queue.clone().lock().unwrap().push(current());
    loop {
        let task = queue.nextTask();
        let results = task.process_this();
        for res in results.iter() {
            queue.addTask(&res);
        }
    }
}
