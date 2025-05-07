use chrono::{DateTime, Local};
use num_cpus;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

#[derive(Clone)]
pub struct Task {
    id: u32,
    name: String,
    call: Arc<Box<dyn Fn() + Send + Sync>>,
    interval: Option<u64>,
}

impl Task {
    pub fn new(name: String, call: Box<dyn Fn() + Send + Sync>) -> Self {
        let id = rand::rng().random::<u32>();
        Task {
            id,
            name,
            call: Arc::new(call),
            interval: None,
        }
    }

    pub fn recurring(name: String, call: Box<dyn Fn() + Send + Sync>, interval: u64) -> Self {
        let id = rand::rng().random::<u32>();
        Task {
            id,
            name,
            call: Arc::new(call),
            interval: Some(interval),
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name && self.interval == other.interval
    }
}
impl Eq for Task {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        other.interval.unwrap().cmp(&self.interval.unwrap())
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task {{ id: {}, name: {}, interval: {:?} }}",
            self.id, self.name, self.interval
        )
    }
}

#[derive(PartialEq, PartialOrd, Eq)]
struct ScheduledTask {
    execution_time: DateTime<Local>,
    task: Task,
}

impl ScheduledTask {
    fn next_event(&self) -> ScheduledTask {
        let mut next_execution_time = self.execution_time;
        if let Some(interval) = self.task.interval {
            next_execution_time = next_execution_time + chrono::Duration::seconds(interval as i64);
        }
        ScheduledTask {
            execution_time: next_execution_time,
            task: self.task.clone(),
        }
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        other.execution_time.cmp(&self.execution_time)
    }
}

pub struct Scheduler {
    receiver: Arc<Mutex<Receiver<Task>>>,
    sender: Sender<Task>,
    running: Arc<AtomicBool>,
    recurring_tasks: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    worker_count: usize,
    worker_threads: Vec<thread::JoinHandle<()>>,
    started: DateTime<Local>,
    last_interval_check: Arc<Mutex<DateTime<Local>>>,
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
            running: Arc::new(AtomicBool::new(false)),
            worker_count: num_cpus::get(),
            worker_threads: Vec::new(),
            recurring_tasks: Arc::new(Mutex::new(BinaryHeap::<ScheduledTask>::new())),
            started: Local::now(),
            last_interval_check: Arc::new(Mutex::new(Local::now())),
        }
    }

    pub fn schedule(&mut self, task: Task) -> Result<(), ScheduleError> {
        if !self.running.load(AtomicOrdering::Relaxed) {
            // Start the worker threads
            println!("Starting worker threads with {} workers", self.worker_count);
            self.start_worker_threads();
            self.start_recurring_worker_threads();

            self.running.store(true, AtomicOrdering::Relaxed);
            self.started = Local::now();
        }

        // Scheduling logic goes here
        println!("Scheduling task: {}", task.name);
        // Send the task to the receiver
        if task.interval.is_some() {
            // If the task is recurring, send it to the recurring receiver
            let interval = task.interval.unwrap();
            let now = Local::now();
            let execution_time = now + chrono::Duration::seconds(interval as i64);
            let mut recurring_tasks = self.recurring_tasks.lock().unwrap();
            println!(
                "Recurring task scheduled {} to run at {:?}",
                task.name, execution_time
            );
            recurring_tasks.push(ScheduledTask {
                execution_time,
                task,
            });
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

    fn start_worker_threads(&mut self) {
        for _ in 0..self.worker_count {
            let receiver_clone = Arc::clone(&self.receiver);
            let worker_thread = thread::spawn(move || {
                loop {
                    let receiver = receiver_clone.lock().unwrap();
                    match receiver.recv_timeout(Duration::from_secs(1)) {
                        Ok(task) => {
                            drop(receiver); // Drop the lock to avoid deadlock
                            if task.interval.is_some() {
                                println!("Recurring task executing: {}", task.name);
                            } else {
                                println!("Task executing: {}", task.name);
                            }
                            (task.call)();
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            drop(receiver);
                            // Just a timeout, check running status and continue
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            println!("Sender closed");
                            break;
                        }
                    }
                }
            });
            self.worker_threads.push(worker_thread);
        }
    }

    fn start_recurring_worker_threads(&mut self) {
        {
            let mut interval = self.last_interval_check.lock().unwrap();
            *interval = Local::now();
        }

        // Clone the references to shared state
        let recurring_tasks = Arc::clone(&self.recurring_tasks);
        let last_interval_check = Arc::clone(&self.last_interval_check);
        let sender = self.sender.clone();
        let running = Arc::clone(&self.running);
        let recurring_worker_thread = thread::spawn(move || {
            loop {
                if !running.load(AtomicOrdering::Relaxed) {
                    println!("Recurring worker thread stopping");
                    return;
                }
                let now = Local::now();

                println!("Recurring worker thread checking tasks at {:?}", now,);

                let sleep_duration = {
                    let recurring_tasks_guard = recurring_tasks.lock().unwrap();
                    if let Some(next_task) = recurring_tasks_guard.peek() {
                        println!(
                            "Next task to run: {} at {:?} and current time is {:?}",
                            next_task.task.name, next_task.execution_time, now
                        );
                        if next_task.execution_time <= now {
                            // Task is already due, process immediately
                            0
                        } else {
                            // Calculate exact time until next task
                            let wait_time = next_task.execution_time.timestamp_millis()
                                - now.timestamp_millis();
                            // Use a minimum sleep to avoid busy waiting
                            std::cmp::max(wait_time, 10 as i64)
                        }
                    } else {
                        // No tasks in queue, check again in 100ms
                        100
                    }
                };

                // Sleep only until the next task is due (or for a short time if no tasks)
                if sleep_duration > 0 {
                    println!("Recurring worker thread sleeping for {} ms", sleep_duration);
                    thread::sleep(Duration::from_millis(sleep_duration as u64));
                    continue;
                }

                // Process all tasks that are due now
                let now = Local::now(); // Update time after sleeping
                let mut tasks_to_reschedule = Vec::new();

                {
                    let mut recurring_tasks_guard = recurring_tasks.lock().unwrap();

                    while let Some(event) = recurring_tasks_guard.peek() {
                        if event.execution_time > now {
                            break;
                        }
                        // Remove the task from the heap
                        let event = recurring_tasks_guard.pop().unwrap();
                        tasks_to_reschedule.push(event);
                    }
                }

                // Send the tasks to the worker threads
                for event in &tasks_to_reschedule {
                    // Enqueue the task to the sender
                    println!(
                        "Recurring task {} due at {:?}, enqueuing at {:?}",
                        event.task.name,
                        &event.execution_time,
                        Local::now(),
                    );

                    if let Err(e) = sender.send(event.task.clone()) {
                        println!("Failed to send task: {}", e);
                    }
                }

                let rescheduled_tasks: Vec<_> = tasks_to_reschedule
                    .iter()
                    .map(|event| event.next_event())
                    .collect();

                if !rescheduled_tasks.is_empty() {
                    let mut recurring_tasks_guard = recurring_tasks.lock().unwrap();
                    for event in rescheduled_tasks {
                        recurring_tasks_guard.push(event);
                    }
                }

                // Update the last interval check time
                {
                    let mut t = last_interval_check.lock().unwrap();
                    *t = Local::now();
                }
            }
        });

        self.worker_threads.push(recurring_worker_thread);
    }

    pub fn stop(&mut self) {
        self.running.store(false, AtomicOrdering::Relaxed);

        let _ = std::mem::replace(&mut self.sender, channel::<Task>().0);

        let threads = std::mem::take(&mut self.worker_threads);

        for thread in threads {
            if let Err(e) = thread.join() {
                eprintln!("Error joining thread: {:?}", e);
            }
        }
        println!("Scheduler stopped");
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.stop();
    }
}
