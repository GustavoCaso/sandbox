use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Task {
    id: u32,
    name: String,
    call: Box<dyn Fn() -> () + Send>,
}

impl Task {
    pub fn new(name: String, call: Box<dyn Fn() -> () + Send>) -> Self {
        let id = rand::rng().random::<u32>();
        Task { id, name, call }
    }
}

pub struct Scheduler {
    receiver: Arc<Mutex<Receiver<Task>>>,
    sender: Sender<Task>,
    running: AtomicBool,
}

pub struct ScheduleError {
    message: String,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScheduleError: {}", self.message)
    }
}

impl std::error::Error for ScheduleError {}

impl Scheduler {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Task>();
        Scheduler {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
            running: AtomicBool::new(false),
        }
    }

    pub fn schedule(&self, task: Task) -> Result<(), ScheduleError> {
        if !self.running.load(Ordering::Relaxed) {
            let receiver_clone = Arc::clone(&self.receiver);
            thread::spawn(move || {
                loop {
                    let receiver = receiver_clone.lock().unwrap();
                    match receiver.recv() {
                        Ok(task) => {
                            drop(receiver); // Drop the lock to avoid deadlock
                            (task.call)();
                        }
                        Err(e) => {
                            println!("Receiver closed: {}", e);
                            break;
                        }
                    }
                }
            });
            self.running.store(true, Ordering::Relaxed);
        }

        // Scheduling logic goes here
        println!("Scheduling task: {}", task.name);
        // Send the task to the receiver
        let result = self.sender.send(task);
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let message = format!("Failed to send task: {}", e);
                return Err(ScheduleError { message });
            }
        }
    }
}
