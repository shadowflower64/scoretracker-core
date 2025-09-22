use crate::hive::job::{self, Job};
use crate::hive::worker::WorkerInfo;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskResult {
    Success(job::Success),
    Error(job::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub uuid: UuidString,
    pub name: String,
    pub comment: Option<String>,
    pub job: Job,
    pub state: TaskState,
    pub request_timestamp: NsTimestamp,
    pub start_timestamp: Option<NsTimestamp>,
    pub worker_info: Option<WorkerInfo>,
    pub finish_timestamp: Option<NsTimestamp>,
    pub result: Option<TaskResult>,
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
            worker_info: None,
            finish_timestamp: None,
            result: None,
        }
    }
}
