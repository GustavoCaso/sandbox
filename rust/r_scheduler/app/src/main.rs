use scheduler::Scheduler;
use scheduler::Task;
use std::sync::{Arc, Mutex};
use std::{thread, time};

fn main() {
    let scheduler = Arc::new(Mutex::new(Scheduler::new()));
    for i in 0..10 {
        let scheduler_clone = Arc::clone(&scheduler);
        thread::spawn(move || {
            let task = Task::new(
                format!("Task {}", i).to_string(),
                Box::new(move || {
                    println!("Executing Task {}", i);
                }),
            );
            let s = scheduler_clone.lock().unwrap();
            let _ = s.schedule(task);
        });
    }

    thread::sleep(time::Duration::from_secs(2));
}
