use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time;

type AsyncTaskFn = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>;

pub struct Scheduler {
    tasks: HashMap<String, (Duration, AsyncTaskFn)>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }

    /// Register a sync closure.
    pub fn register(&mut self, name: impl Into<String>, interval: Duration, f: impl Fn() + Send + Sync + 'static) {
        let f = Arc::new(f);
        self.register_async(name, interval, move || {
            let f = f.clone();
            Box::pin(async move { f() })
        });
    }

    /// Register an async closure.
    pub fn register_async<F, Fut>(&mut self, name: impl Into<String>, interval: Duration, f: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let wrapped: AsyncTaskFn = Arc::new(move || Box::pin(f()) as Pin<Box<dyn Future<Output = ()> + Send>>);
        self.tasks.insert(name.into(), (interval, wrapped));
    }

    pub fn spawn_all(self) -> Vec<JoinHandle<()>> {
        self.tasks.into_values().map(|(interval, f)| {
            tokio::spawn(async move {
                let mut ticker = time::interval(interval);
                loop {
                    ticker.tick().await;
                    f().await;
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
