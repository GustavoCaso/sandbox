use rand::Rng;

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

pub struct Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {}
    }

    pub fn schedule(&self, task: Task) {
        // Scheduling logic goes here
        println!("Scheduling task: {}", task.name);
        (task.call)();
    }
}
