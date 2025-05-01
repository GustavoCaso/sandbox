use scheduler::Scheduler;
use scheduler::Task;
use std::{thread, time};

fn main() {
    let s: Scheduler = Scheduler::new();
    let task = Task::new(
        "Task 1".to_string(),
        Box::new(|| {
            println!("Executing Task 1");
        }),
    );

    let _ = s.schedule(task);
    thread::sleep(time::Duration::from_secs(2));
}
