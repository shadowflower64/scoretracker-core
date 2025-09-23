mod ipc;

use crate::config::Config;
use crate::hive::queue::TaskNotFound;
use crate::hive::task::{Task, TaskResult, TaskState};
use crate::hive::worker::ipc::start_listener_thread;
use crate::util::lockfile;
use crate::{error, info, log_fn_name, success};
use crate::{hive::queue::TaskQueueLock, util::timestamp::NsTimestamp};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpListener};
use std::{io, process};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot read from queue: {0}")]
    CannotReadQueue(lockfile::Error),
    #[error("cannot reopen queue: {0}")]
    CannotReopenQueue(lockfile::Error),
    #[error("cannot write to queue: {0}")]
    CannotWriteQueue(lockfile::Error),
    #[error("cannot update task in queue: {0}")]
    CannotUpdateTask(TaskNotFound),
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub name: String,
    pub pid: u32,
    pub birth_timestamp: NsTimestamp,
    pub address: SocketAddr,
}

#[derive(Debug)]
pub struct Worker {
    info: WorkerInfo,
    config: Config,
}

impl Default for Worker {
    fn default() -> Self {
        let pid = process::id();
        let config = Config::load().expect("invalid configuration");
        Self::new_try_connect(format!("defaultworker{pid}.scoretracker.local"), config).expect("could not bind tcp listener")
    }
}

impl Worker {
    pub fn new(name: String, config: Config, listener: TcpListener) -> Self {
        let address = listener.local_addr().expect("could not get local address of tcp listener");
        start_listener_thread(listener);
        Worker {
            info: WorkerInfo {
                name,
                pid: process::id(),
                birth_timestamp: NsTimestamp::now(),
                address,
            },
            config,
        }
    }

    pub fn new_try_connect(name: String, config: Config) -> Result<Self, io::Error> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        Ok(Self::new(name, config, listener))
    }

    pub fn info(&self) -> WorkerInfo {
        self.info.clone()
    }

    /// Take on the first task from the queue and execute it in the current thead.
    pub fn take_on_task(&self) -> Result<(), Error> {
        log_fn_name!("worker:take_on_task");
        type E = Error;
        let worker_info = Some(&self.info);
        pub const QUEUE_PATH: &str = TaskQueueLock::STANDARD_FILENAME; // todo

        // Read the queue to either add something or take on a task
        let mut queue = TaskQueueLock::read_or_create_new_safe(QUEUE_PATH, worker_info).map_err(E::CannotReadQueue)?;

        if let Some(task_to_do) = queue.top_queued_task_mut() {
            // Take on a task
            task_to_do.state = TaskState::Working;
            task_to_do.start_timestamp = Some(NsTimestamp::now());
            task_to_do.worker_info = Some(self.info());
            // task_to_do.comment = Some(String::from("this job was started by scoretracker-core"));

            let mut task = task_to_do.clone();
            info!("taking on task with uuid: {}", task.uuid.0);

            queue.write_to_file().map_err(E::CannotWriteQueue)?;

            // Drop file lock here to and let other processes access the queue
            let queue = queue.close();

            // Do some task if there is something to do
            self.execute_task(&mut task);

            // Update the queue file again to update the state of the task
            let mut queue = queue.reopen(worker_info).map_err(E::CannotReopenQueue)?;
            queue.update_task(task).map_err(E::CannotUpdateTask)?;
            queue.write_to_file().map_err(E::CannotWriteQueue)?;
        } else {
            info!("no tasks to do");
        }
        Ok(())
    }

    /// Execute a task in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// The task should be marked as "being worked on" before executing this method to prevent other processes from doing the same task.
    /// The queue file should be written to before calling this method.
    ///
    /// The result of the task should also be saved to the queue after this method finishes, so that no data is lost and the task is not done twice.
    fn execute_task(&self, task: &mut Task) {
        log_fn_name!("worker:execute task");
        let worker_info = Some(&self.info);

        match task.job.run(&self.config, worker_info) {
            Ok(success) => {
                success!("task finished successfully: uuid: {} results: {:#?}", task.uuid.0, success);
                task.state = TaskState::Done;
                task.result = Some(TaskResult::Success(success));
            }
            Err(error) => {
                error!("task failed: uuid: {} error: {:?}", task.uuid.0, error);
                task.state = TaskState::Failed;
                task.result = Some(TaskResult::Error(error));
            }
        }
        task.finish_timestamp = Some(NsTimestamp::now());
    }

    /// Execute a task from the queue in the current thread.
    pub fn execute_task_by_uuid(&self, task_uuid: Uuid) -> Result<(), Error> {
        log_fn_name!("worker:execute_task_by_uuid");
        type E = Error;
        let worker_info = Some(&self.info);
        pub const QUEUE_PATH: &str = TaskQueueLock::STANDARD_FILENAME; // todo

        // Read the queue to either add something or take on a task
        let mut queue = TaskQueueLock::read_or_create_new_safe(QUEUE_PATH, worker_info).map_err(E::CannotReadQueue)?;

        // Take on a task
        let task_to_do = queue.get_task_mut(task_uuid).ok_or(E::TaskNotFound(task_uuid))?;
        task_to_do.state = TaskState::Working;
        task_to_do.start_timestamp = Some(NsTimestamp::now());
        task_to_do.worker_info = worker_info.cloned();
        // task_to_do.comment = Some(String::from("this job was started by scoretracker-core"));

        let mut task = task_to_do.clone();
        info!("taking on task with uuid: {}", task.uuid.0);

        queue.write_to_file().map_err(E::CannotWriteQueue)?;

        // Drop file lock here to and let other processes access the queue
        let queue = queue.close();

        // Do some task if there is something to do
        self.execute_task(&mut task);

        // Update the queue file again to update the state of the task
        let mut queue = queue.reopen(worker_info).map_err(E::CannotReopenQueue)?;
        queue.update_task(task).map_err(E::CannotUpdateTask)?;
        queue.write_to_file().map_err(E::CannotWriteQueue)?;
        Ok(())
    }
}
