use std::{sync::Arc, thread};

use anyhow::Result;
use tokio::{sync::mpsc, task};

use crate::db::stats::{self, DbStats, EnvStats};
use heed::Env;

/// Background job types.
pub enum Job {
    Env,
    Db(String),
}

/// Result messages from background jobs.
pub enum JobResult {
    Env(EnvStats),
    Db(String, DbStats),
}

/// Simple job queue backed by a Tokio runtime running in a thread.
pub struct JobQueue {
    sender: Option<mpsc::UnboundedSender<Job>>,
    pub receiver: mpsc::UnboundedReceiver<JobResult>,
    handle: Option<thread::JoinHandle<()>>,
}

impl JobQueue {
    pub fn new(env: Env) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (res_tx, res_rx) = mpsc::unbounded_channel();
        let env = Arc::new(env);
        let env_thread = env.clone();
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(async move {
                while let Some(job) = rx.recv().await {
                    match job {
                        Job::Env => {
                            let e = env_thread.clone();
                            match task::spawn_blocking(move || stats::env_stats(&e)).await {
                                Ok(stats) => {
                                    let _ = res_tx.send(JobResult::Env(stats));
                                }
                                Err(e) => {
                                    log::error!("Failed to calculate environment stats: {}", e);
                                }
                            }
                        }
                        Job::Db(name) => {
                            let e = env_thread.clone();
                            let name_clone = name.clone();
                            match task::spawn_blocking(move || stats::db_stats(&e, &name_clone))
                                .await
                            {
                                Ok(Ok(stats)) => {
                                    let _ = res_tx.send(JobResult::Db(name, stats));
                                }
                                Ok(Err(e)) => {
                                    log::warn!(
                                        "Failed to calculate stats for database '{}': {}",
                                        name,
                                        e
                                    );
                                }
                                Err(e) => {
                                    log::error!("Task join error for database '{}': {}", name, e);
                                }
                            }
                        }
                    }
                }
            });
        });
        Self {
            sender: Some(tx),
            receiver: res_rx,
            handle: Some(handle),
        }
    }

    pub fn request_env_stats(&self) -> Result<()> {
        if let Some(sender) = &self.sender {
            sender.send(Job::Env)?;
        }
        Ok(())
    }

    pub fn request_db_stats(&self, db: String) -> Result<()> {
        if let Some(sender) = &self.sender {
            sender.send(Job::Db(db))?;
        }
        Ok(())
    }
}

impl Drop for JobQueue {
    fn drop(&mut self) {
        // Drop the sender first to close the request channel
        let _ = self.sender.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
