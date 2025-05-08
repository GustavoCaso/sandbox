use chrono::prelude::*;
use scheduler::Scheduler;
use scheduler::Task;
use std::{thread, time};

fn main() {
    let mut scheduler = Scheduler::new();
    for i in 0..10 {
        let task = Task::new(
            format!("Task {}", i).to_string(),
            None,
            Box::new(move || {
                println!("Executing Task {}", i);
            }),
        );

        let recurring_task = Task::new(
            format!("Recurring Task {}", i).to_string(),
            Some(5 + i),
            Box::new(move || {
                println!(
                    "Executing Recurring Task {} at {}",
                    i,
                    Local::now().to_rfc2822()
                );
            }),
        );

        let _ = scheduler.schedule(task);
        let _ = scheduler.schedule(recurring_task);
    }

    thread::sleep(time::Duration::from_secs(200));
}
