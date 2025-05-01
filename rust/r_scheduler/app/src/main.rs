use scheduler::Scheduler;
use scheduler::Task;

fn main() {
    let s = Scheduler::new();
    let task = Task::new(
        "Task 1".to_string(),
        Box::new(|| {
            println!("Executing Task 1");
        }),
    );

    s.schedule(task);
}
