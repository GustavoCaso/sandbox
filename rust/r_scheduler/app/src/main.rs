use chrono::prelude::*;
use scheduler::Scheduler;
use scheduler::Task;
use std::{thread, time};

fn main() {
    let mut scheduler = Scheduler::new();
    for i in 0..10 {
        let task = Task::new(
            format!("Task {}", i).to_string(),
            Box::new(move || {
                println!("Executing Task {}", i);
            }),
        );

        let recurring_task = Task::recurring(
            format!("Recurring Task {}", i).to_string(),
            Box::new(move || {
                println!(
                    "Executing Recurring Task {} at {}",
                    i,
                    Local::now().to_rfc2822()
                );
            }),
            5 + i,
        );

        let _ = scheduler.schedule(task);
        let _ = scheduler.schedule(recurring_task);
    }

    thread::sleep(time::Duration::from_secs(200));
}
