use chrono::{DateTime, Local};
use num_cpus;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

#[derive(Clone)]
pub struct Task {
    id: u32,
    name: String,
    call: Arc<Box<dyn Fn() -> () + Send + Sync>>,
    interval: Option<u64>,
}

impl Task {
    pub fn new(name: String, call: Box<dyn Fn() -> () + Send + Sync>) -> Self {
        let id = rand::rng().random::<u32>();
        Task {
            id,
            name,
            call: Arc::new(call),
            interval: None,
        }
    }

    pub fn recurring(name: String, call: Box<dyn Fn() -> () + Send + Sync>, interval: u64) -> Self {
        let id = rand::rng().random::<u32>();
        Task {
            id,
            name,
            call: Arc::new(call),
            interval: Some(interval),
        }
    }
}

pub struct Scheduler {
    receiver: Arc<Mutex<Receiver<Task>>>,
    sender: Sender<Task>,
    running: AtomicBool,
    recurring_tasks: Arc<Mutex<Vec<(Instant, Task)>>>,
    worker_count: usize,
    started: Instant,
    last_interval_check: Arc<Mutex<Instant>>,
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

// Convert an Instant to a human-readable string
fn format_instant(instant: &Instant, start: &Instant) -> String {
    // Calculate duration since start
    let duration = instant.duration_since(*start);

    // Convert to current time (approximation)
    let now = SystemTime::now();
    let target_time = now + duration;

    // Convert to DateTime for formatting
    let datetime: DateTime<Local> = target_time.into();

    // Format as a readable string
    datetime.format("%H:%M:%S").to_string()
}

impl Scheduler {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Task>();
        Scheduler {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
            running: AtomicBool::new(false),
            worker_count: num_cpus::get(),
            recurring_tasks: Arc::new(Mutex::new(Vec::new())),
            started: Instant::now(),
            last_interval_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn schedule(&mut self, task: Task) -> Result<(), ScheduleError> {
        if !self.running.load(Ordering::Relaxed) {
            // Start the worker threads
            println!("Starting worker threads with {} workers", self.worker_count);
            self.start_worker_threads();
            self.start_recurring_worker_threads();

            self.running.store(true, Ordering::Relaxed);
            self.started = Instant::now();
        }

        // Scheduling logic goes here
        println!("Scheduling task: {}", task.name);
        // Send the task to the receiver
        if task.interval.is_some() {
            // If the task is recurring, send it to the recurring receiver
            let interval = task.interval.unwrap();
            let now = Instant::now();
            let next_run = now + Duration::from_secs(interval);
            let mut recurring_tasks = self.recurring_tasks.lock().unwrap();
            println!(
                "Recurring task scheduled {} to run at {:?}",
                task.name,
                format_instant(&next_run, &self.started)
            );
            recurring_tasks.push((next_run, task));
            Ok(())
        } else {
            // If the task is not recurring, send it to the worker threads
            match self.sender.send(task) {
                Ok(_) => {
                    println!("Task sent to worker thread");
                    Ok(())
                }
                Err(e) => {
                    println!("Failed to send task: {}", e);
                    return Err(ScheduleError {
                        message: "Failed to send task".to_string(),
                    });
                }
            }
        }
    }

    fn start_worker_threads(&self) {
        for _ in 0..self.worker_count {
            let receiver_clone = Arc::clone(&self.receiver);
            thread::spawn(move || {
                loop {
                    let receiver = receiver_clone.lock().unwrap();
                    match receiver.recv() {
                        Ok(task) => {
                            drop(receiver); // Drop the lock to avoid deadlock
                            if task.interval.is_some() {
                                println!("Recurring task executing: {}", task.name);
                            } else {
                                println!("Task executing: {}", task.name);
                            }
                            (task.call)();
                        }
                        Err(e) => {
                            println!("Receiver closed: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }

    fn start_recurring_worker_threads(&self) {
        {
            let mut interval = self.last_interval_check.lock().unwrap();
            *interval = Instant::now();
        }

        // Clone the references to shared state
        let recurring_tasks = Arc::clone(&self.recurring_tasks);
        let last_interval_check = Arc::clone(&self.last_interval_check);
        let sender = self.sender.clone();
        let started = self.started.clone();
        thread::spawn(move || {
            loop {
                let to_enqueue_taks = {
                    let mut recurring_tasks_guard = recurring_tasks.lock().unwrap();

                    let (to_enqueue, to_keep): (Vec<_>, Vec<_>) = recurring_tasks_guard
                        .drain(..)
                        .partition(|(time, _)| *time <= Instant::now());

                    recurring_tasks_guard.extend(to_keep);

                    to_enqueue
                };

                for (_, task) in to_enqueue_taks {
                    let now = Instant::now();
                    let next_run = now + Duration::from_secs(task.interval.unwrap());
                    println!(
                        "Recurring task re-scheduled {} to run at {:?}",
                        task.name,
                        format_instant(&next_run, &started)
                    );
                    {
                        let mut recurring_tasks_guard = recurring_tasks.lock().unwrap();
                        recurring_tasks_guard.push((next_run, task.clone()));
                    }
                    match sender.send(task) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Failed to send task: {}", e);
                        }
                    }
                }

                // Update the last interval check time
                {
                    let mut t = last_interval_check.lock().unwrap();
                    *t = Instant::now();
                }
                thread::sleep(Duration::from_secs(1)); // Sleep for a second before checking again
            }
        });
    }
}
