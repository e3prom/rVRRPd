//! threads pool module
//! This module implement the thread pool.
use super::*;

// concurrency
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

// channels
use std::sync::mpsc;

// debugging
use crate::debug::Verbose;

// finite state machine
use fsm::{fsm_run, Event};

/// ThreadPool Structure
pub struct ThreadPool {
    workers: Vec<Worker>,
}

// ThreadPool Implementation
impl ThreadPool {
    // new() method
    // Create a new Thread Pool
    pub fn new(vrouters: &Vec<Arc<RwLock<VirtualRouter>>>, debug: &Verbose) -> ThreadPool {
        // verify the vector is not empty and doesn't exceed 1024 virtual routers
        assert!(vrouters.len() > 0 && vrouters.len() < 1024);

        // built a fixed-size vector of workers
        let mut workers = Vec::with_capacity(vrouters.len());

        // creating individual workers for every virtual routers
        for (id, vr) in vrouters.iter().enumerate() {
            // acquire read lock on vr
            let vro = vr.read().unwrap();
            // create new worker
            workers.push(Worker::new(id, Arc::clone(&vr), vro.parameters.fd(), debug));
        }

        ThreadPool { workers }
    }
    // startup() method
    // Send startup event to every worker threads
    pub fn startup(&self, vrouters: &Vec<Arc<RwLock<VirtualRouter>>>, debug: &Verbose) {
        for (id, vr) in vrouters.iter().enumerate() {
            // acquire read lock on vr
            let vr = vr.read().unwrap();
            // print debugging information
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_THREAD,
                format!("sending Startup event to worker threads"),
            );
            match vr.parameters.notification() {
                Some(tx) => tx.lock().unwrap().send(Event::Startup).unwrap(),
                None => eprintln!("error(thread): cannot send Startup event for thread {}, channel does not exist", id),
            }
        }
    }
    // drop() method
    // Custom destructor function for the Thread pool
    pub fn drop(&mut self, vrouters: &Vec<Arc<RwLock<VirtualRouter>>>, debug: &Verbose) {
        print_debug(
            debug,
            DEBUG_LEVEL_LOW,
            DEBUG_SRC_THREAD,
            format!("signaling workers to shut down"),
        );
        // send Shutdown/Terminate events to all workers
        for (id, vr) in vrouters.iter().enumerate() {
            // acquire read lock on vr
            let vr = vr.read().unwrap();
            match vr.parameters.notification() {
                Some(tx) => {
                    // send Shutdown event
                    tx.lock().unwrap().send(Event::Shutdown).unwrap();
                    // send Terminate event
                    tx.lock().unwrap().send(Event::Terminate).unwrap();
                }
                None => eprintln!(
                    "error(thread): cannot send Terminate event for {}, channel does not exist",
                    id
                ),
            }
        }

        // waiting for threads
        for worker in &mut self.workers {
            // print debugging information
            print_debug(
                debug,
                DEBUG_LEVEL_HIGH,
                DEBUG_SRC_THREAD,
                format!("waiting for thread {} to exit...", worker.id),
            );
            // take the thread out of the worker stucture and leave a None
            if let Some(thread) = worker.thread.take() {
                // Wait for the thread to finish
                thread.join().unwrap();
            }
        }
    }
}

// new Trait FnBox to take ownership of a Self stored in a Box<T>
pub trait FnBox {
    fn call_box(self: Box<Self>);
}

// new FnBox trait for any type F having the Trait FnOnce()
impl<F: FnOnce()> FnBox for F {
    // every FnOnce() closures can now call the .call_box() method
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// Worker Structure
pub struct Worker {
    id: usize,
    // we wrap thread::JoinHandle in a Option<T> so we can
    // consume the thread later when calling .join().
    thread: Option<thread::JoinHandle<()>>,
}

// Worker Implementation
impl Worker {
    // new() method
    fn new(id: usize, vr: Arc<RwLock<VirtualRouter>>, sockfd: i32, debug: &Verbose) -> Worker {
        // creating a pair of sender and receiver channels
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let sender = Arc::new(Mutex::new(sender));

        // clone VR for worker thread
        let worker_vr = Arc::clone(&vr);

        // clone event channels
        let worker_tx = Arc::clone(&sender);
        let worker_rx = Arc::clone(&receiver);

        // clone debug
        let debug = debug.clone();

        // create worker thread
        let worker_thread = thread::spawn(move || {
            // print debugging information
            print_debug(
                &debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_THREADP,
                format!("spawning worker thread {}", id),
            );
            fsm_run(id, &worker_tx, &worker_rx, &worker_vr, sockfd, &debug);
        });

        Worker {
            id,
            thread: Some(worker_thread),
        }
    }
}
