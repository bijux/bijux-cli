use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::backend_cluster::GenericBatchExecutorContract;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FakeBatchJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FakeBatchJobRecord {
    pub job_id: String,
    pub run_id: String,
    pub node_id: String,
    pub status: FakeBatchJobStatus,
    pub exit_code: Option<i32>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FakeBatchExecutorContract {
    pub submit_api: String,
    pub poll_api: String,
    pub cancel_api: String,
    pub supported_states: Vec<String>,
    pub failure_mapping: BTreeMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct FakeBatchExecutor {
    next_job: u64,
    jobs: BTreeMap<String, FakeBatchJobRecord>,
}

impl FakeBatchExecutor {
    pub fn submit(&mut self, run_id: &str, node_id: &str) -> String {
        self.next_job = self.next_job.saturating_add(1);
        let job_id = format!("fake-batch-{}", self.next_job);
        self.jobs.insert(
            job_id.clone(),
            FakeBatchJobRecord {
                job_id: job_id.clone(),
                run_id: run_id.to_string(),
                node_id: node_id.to_string(),
                status: FakeBatchJobStatus::Queued,
                exit_code: None,
                diagnostics: Vec::new(),
            },
        );
        job_id
    }

    pub fn transition(
        &mut self,
        job_id: &str,
        next: FakeBatchJobStatus,
    ) -> Result<FakeBatchJobRecord, String> {
        let record =
            self.jobs.get_mut(job_id).ok_or_else(|| format!("unknown fake batch job {job_id}"))?;
        let current = record.status.clone();
        let legal = matches!(
            (&current, &next),
            (FakeBatchJobStatus::Queued, FakeBatchJobStatus::Running)
                | (FakeBatchJobStatus::Queued, FakeBatchJobStatus::Cancelled)
                | (FakeBatchJobStatus::Running, FakeBatchJobStatus::Completed)
                | (FakeBatchJobStatus::Running, FakeBatchJobStatus::Failed)
                | (FakeBatchJobStatus::Running, FakeBatchJobStatus::Cancelled)
        );
        if !legal {
            return Err(format!("illegal fake batch transition {:?} -> {:?}", current, next));
        }
        record.status = next;
        Ok(record.clone())
    }

    pub fn complete_failure(
        &mut self,
        job_id: &str,
        exit_code: i32,
        diagnostic: &str,
    ) -> Result<FakeBatchJobRecord, String> {
        let record =
            self.jobs.get_mut(job_id).ok_or_else(|| format!("unknown fake batch job {job_id}"))?;
        if !matches!(record.status, FakeBatchJobStatus::Running) {
            return Err("fake batch failure can only be recorded from running".to_string());
        }
        record.status = FakeBatchJobStatus::Failed;
        record.exit_code = Some(exit_code);
        record.diagnostics.push(diagnostic.to_string());
        Ok(record.clone())
    }

    pub fn cancel(&mut self, job_id: &str, diagnostic: &str) -> Result<FakeBatchJobRecord, String> {
        let record =
            self.jobs.get_mut(job_id).ok_or_else(|| format!("unknown fake batch job {job_id}"))?;
        if matches!(
            record.status,
            FakeBatchJobStatus::Completed
                | FakeBatchJobStatus::Failed
                | FakeBatchJobStatus::Cancelled
        ) {
            return Err("fake batch cancel requires a non-terminal job".to_string());
        }
        record.status = FakeBatchJobStatus::Cancelled;
        record.diagnostics.push(diagnostic.to_string());
        Ok(record.clone())
    }

    pub fn snapshot(&self, job_id: &str) -> Option<FakeBatchJobRecord> {
        self.jobs.get(job_id).cloned()
    }
}

pub fn fake_batch_executor_contract() -> FakeBatchExecutorContract {
    FakeBatchExecutorContract {
        submit_api: "submit(run_id,node_id) -> job_id".to_string(),
        poll_api: "snapshot(job_id) -> status".to_string(),
        cancel_api: "cancel(job_id, diagnostic)".to_string(),
        supported_states: vec![
            "queued".to_string(),
            "running".to_string(),
            "completed".to_string(),
            "failed".to_string(),
            "cancelled".to_string(),
        ],
        failure_mapping: BTreeMap::from([
            ("failed".to_string(), "execution".to_string()),
            ("cancelled".to_string(), "cancelled".to_string()),
            ("queued".to_string(), "pending".to_string()),
        ]),
    }
}

pub fn fake_batch_backend_reference() -> GenericBatchExecutorContract {
    GenericBatchExecutorContract {
        platform_name: "fake-batch".to_string(),
        submit_api: "submit".to_string(),
        poll_api: "snapshot".to_string(),
        cancel_api: "cancel".to_string(),
    }
}
