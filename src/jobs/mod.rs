use std::thread;

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
    sender: mpsc::UnboundedSender<Job>,
    pub receiver: mpsc::UnboundedReceiver<JobResult>,
}

impl JobQueue {
    pub fn new(env: Env) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (res_tx, res_rx) = mpsc::unbounded_channel();
        let env_thread = env.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(async move {
                while let Some(job) = rx.recv().await {
                    match job {
                        Job::Env => {
                            let e = env_thread.clone();
                            let stats = task::spawn_blocking(move || stats::env_stats(&e))
                                .await
                                .unwrap();
                            let _ = res_tx.send(JobResult::Env(stats));
                        }
                        Job::Db(name) => {
                            let e = env_thread.clone();
                            let name_clone = name.clone();
                            let res =
                                task::spawn_blocking(move || stats::db_stats(&e, &name_clone))
                                    .await
                                    .unwrap();
                            if let Ok(s) = res {
                                let _ = res_tx.send(JobResult::Db(name, s));
                            }
                        }
                    }
                }
            });
        });
        Self {
            sender: tx,
            receiver: res_rx,
        }
    }

    pub fn request_env_stats(&self) -> Result<()> {
        Ok(self.sender.send(Job::Env)?)
    }

    pub fn request_db_stats(&self, db: String) -> Result<()> {
        Ok(self.sender.send(Job::Db(db))?)
    }
}
