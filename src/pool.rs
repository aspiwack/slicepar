//! This module implements a minimal thread pool with the ability to
//! "join", that is to terminate the pool when all threads are
//! idle. See also the [threadpool
//! crate](https://github.com/rust-lang/threadpool) for a more serious
//! implementation (but lacking join at the moment), and inspiration
//! to this file.

// temp while developping lib.
#![allow(dead_code,unused_imports)]

use std::sync::{Mutex,Condvar,Arc};
use std::thread;
use std::thread::JoinHandle;
use queue::Queue;

// To call boxed functions. I don't really understand why it works, I
// just took it from the `threadpool` crate.
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// A simplified semaphore to hold the number of idle threads. The
/// counter is assumed to never go negative.
struct Counter {
    count: Mutex<usize>,
    max_count: usize,
    all_idle:Condvar,
}

impl Counter {

    fn new (count:usize) -> Counter {
        let mcount = Mutex::new(count);
        Counter {
            count: mcount,
            max_count: count,
            all_idle: Condvar::new(),
        }
    }

    fn activate(&self) {
        // For simplicity panics when the lock is poisoined.
        // Should be safe in this limited use case.
        let mut count = self.count.lock().ok().unwrap();
        assert!(*count > 0);
        *count -= 1;
    }

    fn go_idle(&self) {
        // For simplicity panics when the lock is poisoined.
        // Should be safe in this limited use case.
        let mut count = self.count.lock().ok().unwrap();
        *count += 1;
        if *count == self.max_count { self.all_idle.notify_all(); }
    }

    fn has_all_idle(&self) -> bool {
        // For simplicity panics when the lock is poisoined.
        // Should be safe in this limited use case.
        let count = self.count.lock().ok().unwrap();
        *count == self.max_count
    }

}

pub type Job = Box<FnBox>;

struct AbsThreadPool<Q:Queue<Job>> {
    channel: Arc<Mutex<Q>>,
    threads: Vec<JoinHandle<()>>,
    idle_threads: Arc<Counter>, // The mutex in `idle_threads` is
                                // always aquired inside the mutex in
                                // `channel`. It is fragile/error
                                // prone, there should probably be a
                                // single mutex. It would be slightly
                                // less modular, though.
    notify_job: Arc<Condvar>,
}

impl<Q:Queue<Job>+Send+'static> AbsThreadPool<Q> {
    // potential improvement: having a min-size and a max-size to
    // allocate more threads when there are a lot of long tasks
    pub fn new(nthreads:usize,q:Q) -> AbsThreadPool<Q> {
        assert!(nthreads >= 1);
        let q = Arc::new(Mutex::new(q));
        let mut threads = Vec::new();
        let notify = Arc::new(Condvar::new());
        let counter = Arc::new(Counter::new(nthreads));

        for _ in 0..nthreads {
            threads.push(spawn_in_pool(q.clone() , notify.clone() , counter.clone() ));
        };

        AbsThreadPool{
            channel: q,
            threads: threads,
            idle_threads: counter,
            notify_job: notify,
        }
    }

    pub fn execute<F>(&self, f:F) where F:FnOnce()->() + 'static {
        let job = Box::new(f);
        let mut chan = self.channel.lock().ok().unwrap();
        chan.push(job);
        self.notify_job.notify_all();
    }

    pub fn join(&self) {
        let mut lock = self.channel.lock().ok().unwrap();
        loop {
            if lock.is_empty() && self.idle_threads.has_all_idle() {
                break;
            }
            else {
                lock = self.idle_threads.all_idle.wait(lock).ok().unwrap();
            }
        }
    }
}

fn spawn_in_pool<Q:Queue<Job>+Send+'static> (rcv:Arc<Mutex<Q>>,notify:Arc<Condvar>,idle_threads:Arc<Counter>) -> JoinHandle<()> {
    thread::spawn(move || {
       loop {
           let job = {
               // Aquire access to the queue. For simplicity assumes
               // that mutex is not poisoined. If the queue is empty,
               // release mutex and wait for the queue to be filled
               // again.
               let mut lock = rcv.lock().ok().unwrap();
               match lock.pop() {
                   Some(job) => {
                       idle_threads.activate();
                       job
                   },
                   None => {
                       let _ = notify.wait(lock);
                       continue; // releases then reaquires the lock
                                 // maybe it would be more efficient
                                 // with a `while let` rather than a
                                 // `match` and a `continue`.
                   }
               }
           };

           job.call_box();
           idle_threads.go_idle();
       };
    })
}
