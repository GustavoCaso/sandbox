use core::panic::PanicMessage;
use rand::Rng;
use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

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
    receiver: Receiver<Task>,
    sender: Sender<Task>,
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
            receiver: rx,
        }
    }

    pub fn schedule(&self, task: Task) -> Result<(), ScheduleError> {
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
