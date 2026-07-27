use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot;

pub struct IndexScheduler {
    debounce: std::time::Duration,
    cancel_tx: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl IndexScheduler {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce: std::time::Duration::from_millis(debounce_ms),
            cancel_tx: Mutex::new(HashMap::new()),
        }
    }

    pub fn submit<F>(&self, job_type: &str, rebuild_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        match self.cancel_tx.lock() {
            Ok(mut map) => {
                if let Some(tx) = map.remove(job_type) {
                    let _ = tx.send(());
                }
                let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
                map.insert(job_type.to_string(), tx);
                let debounce = self.debounce;
                tokio::spawn(async move {
                    tokio::select! {
                        _ = tokio::time::sleep(debounce) => {
                            rebuild_fn();
                        }
                        _ = &mut rx => {
                        }
                    }
                });
            }
            Err(poisoned) => {
                tracing::error!(
                    "IndexScheduler cancel_tx mutex poisoned for job '{}': {}",
                    job_type,
                    poisoned
                );
            }
        }
    }
}
