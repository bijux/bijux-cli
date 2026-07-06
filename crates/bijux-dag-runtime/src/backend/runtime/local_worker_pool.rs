use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub struct LocalWorkerExecution<T> {
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub result: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalWorkerAssignment {
    pub worker_id: usize,
    pub node_id: String,
}

#[derive(Debug)]
pub struct LocalWorkerCompletion<T> {
    pub worker_id: usize,
    pub node_id: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub result: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalWorkerState {
    Idle,
    Running { node_id: String },
    CancelRequested { node_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalWorkerStatus {
    pub worker_id: usize,
    pub state: LocalWorkerState,
}

pub type LocalWorkerJob<T> = Box<dyn FnOnce() -> LocalWorkerExecution<T> + Send + 'static>;

enum LocalWorkerCommand<T> {
    Run { node_id: String, job: LocalWorkerJob<T> },
    Shutdown,
}

enum LocalWorkerMessage<T> {
    Completed(LocalWorkerCompletion<T>),
    Failed { worker_id: usize, node_id: String, message: String },
}

struct LocalWorkerHandle<T> {
    worker_id: usize,
    state: LocalWorkerState,
    command_tx: Sender<LocalWorkerCommand<T>>,
    join_handle: Option<JoinHandle<()>>,
}

pub struct LocalWorkerPool<T: Send + 'static> {
    cancellation_requested: bool,
    completion_rx: Receiver<LocalWorkerMessage<T>>,
    workers: Vec<LocalWorkerHandle<T>>,
}

impl<T: Send + 'static> LocalWorkerPool<T> {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let (completion_tx, completion_rx) = mpsc::channel();
        let mut workers = Vec::with_capacity(capacity);
        for worker_id in 0..capacity {
            let (command_tx, command_rx) = mpsc::channel();
            let completion_tx = completion_tx.clone();
            let join_handle =
                thread::spawn(move || local_worker_main(worker_id, command_rx, completion_tx));
            workers.push(LocalWorkerHandle {
                worker_id,
                state: LocalWorkerState::Idle,
                command_tx,
                join_handle: Some(join_handle),
            });
        }
        drop(completion_tx);
        Self { cancellation_requested: false, completion_rx, workers }
    }

    pub fn capacity(&self) -> usize {
        self.workers.len()
    }

    pub fn available_workers(&self) -> usize {
        self.workers.iter().filter(|worker| matches!(worker.state, LocalWorkerState::Idle)).count()
    }

    pub fn has_running(&self) -> bool {
        self.workers.iter().any(|worker| {
            matches!(
                worker.state,
                LocalWorkerState::Running { .. } | LocalWorkerState::CancelRequested { .. }
            )
        })
    }

    pub fn request_cancellation(&mut self) {
        self.cancellation_requested = true;
        for worker in &mut self.workers {
            if let LocalWorkerState::Running { node_id } = &worker.state {
                worker.state = LocalWorkerState::CancelRequested { node_id: node_id.clone() };
            }
        }
    }

    pub fn status(&self) -> Vec<LocalWorkerStatus> {
        self.workers
            .iter()
            .map(|worker| LocalWorkerStatus {
                worker_id: worker.worker_id,
                state: worker.state.clone(),
            })
            .collect()
    }

    pub fn submit(
        &mut self,
        node_id: String,
        job: LocalWorkerJob<T>,
    ) -> Result<LocalWorkerAssignment, String> {
        if self.cancellation_requested {
            return Err("local worker pool is closed to new submissions".to_string());
        }
        let worker = self
            .workers
            .iter_mut()
            .find(|worker| matches!(worker.state, LocalWorkerState::Idle))
            .ok_or_else(|| "no idle local worker available".to_string())?;
        worker
            .command_tx
            .send(LocalWorkerCommand::Run { node_id: node_id.clone(), job })
            .map_err(|_| format!("local worker {} is unavailable", worker.worker_id))?;
        worker.state = LocalWorkerState::Running { node_id: node_id.clone() };
        Ok(LocalWorkerAssignment { worker_id: worker.worker_id, node_id })
    }

    pub fn wait_for_completion(&mut self) -> Result<LocalWorkerCompletion<T>, String> {
        let message = self
            .completion_rx
            .recv()
            .map_err(|_| "local worker completion channel closed".to_string())?;
        match message {
            LocalWorkerMessage::Completed(completion) => {
                self.mark_worker_idle(completion.worker_id);
                Ok(completion)
            }
            LocalWorkerMessage::Failed { worker_id, node_id, message } => {
                self.mark_worker_idle(worker_id);
                Err(format!("{message} for node '{node_id}'"))
            }
        }
    }

    fn mark_worker_idle(&mut self, worker_id: usize) {
        if let Some(worker) = self.workers.iter_mut().find(|worker| worker.worker_id == worker_id) {
            worker.state = LocalWorkerState::Idle;
        }
    }
}

impl<T: Send + 'static> Drop for LocalWorkerPool<T> {
    fn drop(&mut self) {
        for worker in &self.workers {
            let _ = worker.command_tx.send(LocalWorkerCommand::Shutdown);
        }
        for worker in &mut self.workers {
            if let Some(join_handle) = worker.join_handle.take() {
                let _ = join_handle.join();
            }
        }
    }
}

fn local_worker_main<T: Send + 'static>(
    worker_id: usize,
    command_rx: Receiver<LocalWorkerCommand<T>>,
    completion_tx: Sender<LocalWorkerMessage<T>>,
) {
    while let Ok(command) = command_rx.recv() {
        match command {
            LocalWorkerCommand::Run { node_id, job } => {
                let message = match catch_unwind(AssertUnwindSafe(job)) {
                    Ok(execution) => LocalWorkerMessage::Completed(LocalWorkerCompletion {
                        worker_id,
                        node_id,
                        started_unix_ms: execution.started_unix_ms,
                        finished_unix_ms: execution.finished_unix_ms,
                        result: execution.result,
                    }),
                    Err(_) => LocalWorkerMessage::Failed {
                        worker_id,
                        node_id,
                        message: format!("local worker {worker_id} panicked"),
                    },
                };
                if completion_tx.send(message).is_err() {
                    break;
                }
            }
            LocalWorkerCommand::Shutdown => break,
        }
    }
}
