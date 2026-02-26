use std::collections::HashMap;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time;

type TaskFn = Box<dyn Fn() + Send + Sync + 'static>;

pub struct Scheduler {
    tasks: HashMap<String, (Duration, TaskFn)>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }

    pub fn register(&mut self, name: impl Into<String>, interval: Duration, f: impl Fn() + Send + Sync + 'static) {
        self.tasks.insert(name.into(), (interval, Box::new(f)));
    }

    pub fn spawn_all(self) -> Vec<JoinHandle<()>> {
        self.tasks.into_values().map(|(interval, f)| {
            tokio::spawn(async move {
                let mut ticker = time::interval(interval);
                loop {
                    ticker.tick().await;
                    f();
                }
            })
        }).collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
