use std::{any::Any, collections::HashMap, future::Future, pin::Pin, time::Duration};

use tokio::{sync::mpsc, task::JoinHandle};
use voice_client::ToolCall;

pub(super) type JobId = String;
pub(crate) type JobResult = anyhow::Result<Box<dyn Any + Send>>;
pub(super) type Work = Pin<Box<dyn Future<Output = JobResult> + Send>>;

type Completion = (JobId, u64, Result<JobResult, tokio::time::error::Elapsed>);

struct Running {
    generation: u64,
    task: JoinHandle<()>,
    reply: Option<ToolCall>,
}

pub(super) enum Internal {
    JobDone(JobId, JobResult, Option<ToolCall>),
    Timeout(JobId, Option<ToolCall>),
}

pub(super) struct Jobs {
    running: HashMap<JobId, Running>,
    tx: mpsc::UnboundedSender<Completion>,
    rx: mpsc::UnboundedReceiver<Completion>,
    generation: u64,
}

impl Jobs {
    pub(super) fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            running: HashMap::new(),
            tx,
            rx,
            generation: 0,
        }
    }

    pub(super) fn cancel(&mut self, id: &str) -> Option<ToolCall> {
        self.running.remove(id).and_then(|job| {
            job.task.abort();
            job.reply
        })
    }

    pub(super) fn spawn(
        &mut self,
        job: JobId,
        work: Work,
        reply: Option<ToolCall>,
        timeout: Duration,
    ) -> Option<ToolCall> {
        let previous = self.running.remove(&job).and_then(|old| {
            old.task.abort();
            old.reply
        });
        self.generation += 1;
        let generation = self.generation;
        let tx = self.tx.clone();
        let id = job.clone();
        let task = tokio::spawn(async move {
            let result = tokio::time::timeout(timeout, work).await;
            let _ = tx.send((id, generation, result));
        });
        self.running.insert(
            job,
            Running {
                generation,
                task,
                reply,
            },
        );
        previous
    }

    pub(super) async fn next(&mut self) -> Internal {
        loop {
            let (id, generation, result) = self
                .rx
                .recv()
                .await
                .expect("jobs sender lives with receiver");
            if self
                .running
                .get(&id)
                .is_none_or(|job| job.generation != generation)
            {
                continue;
            }
            let running = self.running.remove(&id).expect("matching job exists");
            return match result {
                Ok(result) => Internal::JobDone(id, result, running.reply),
                Err(_) => Internal::Timeout(id, running.reply),
            };
        }
    }
}

impl Drop for Jobs {
    fn drop(&mut self) {
        for job in self.running.values() {
            job.task.abort();
        }
    }
}

#[cfg(test)]
#[path = "jobs_tests.rs"]
mod tests;
