use crate::hive::job::Job;
use crate::util::timestamp::NsTimestamp;
use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    #[default]
    Queued,
    Working,
    Done,
    Failed,
}

pub type TaskResults = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub uuid: UuidString,
    pub name: String,
    pub comment: Option<String>,
    pub job: Job,
    pub state: TaskState,
    pub request_timestamp: NsTimestamp,
    pub start_timestamp: Option<NsTimestamp>,
    pub worker_name: Option<String>,
    pub worker_pid: Option<u32>,
    pub worker_birth_timestamp: Option<NsTimestamp>,
    pub finish_timestamp: Option<NsTimestamp>,
    pub results: Option<TaskResults>,
}

impl Task {
    pub fn new(name: String, job: Job) -> Self {
        Self {
            uuid: Uuid::new_v4().into(),
            name,
            comment: None,
            job,
            state: TaskState::default(),
            request_timestamp: NsTimestamp::now(),
            start_timestamp: None,
            worker_name: None,
            worker_pid: None,
            worker_birth_timestamp: None,
            finish_timestamp: None,
            results: None,
        }
    }
}
