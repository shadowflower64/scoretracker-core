use crate::hive::queue::TaskNotFound;
use crate::hive::task::{Task, TaskState};
use crate::util::lockfile;
use crate::{error, info, log_fn_name, success};
use crate::{hive::queue::TaskQueue, util::timestamp::NsTimestamp};
use std::process;
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

pub struct Worker {
    worker_name: String,
    pid: u32,
    birth_timestamp: NsTimestamp,
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            worker_name: "default_worker.local".to_string(),
            pid: process::id(),
            birth_timestamp: NsTimestamp::now(),
        }
    }

    /// Take on the first task from the queue and execute it in the current thead.
    pub fn take_on_task(&self) -> Result<(), Error> {
        log_fn_name!("worker:take_on_task");
        type E = Error;
        pub const QUEUE_PATH: &str = "test_queue.jsonl"; // todo

        // Read the queue to either add something or take on a task
        let mut queue = TaskQueue::read_or_create_new_safe(QUEUE_PATH).map_err(E::CannotReadQueue)?;

        if let Some(task_todo) = queue.top_queued_task_mut() {
            // Take on a task
            task_todo.state = TaskState::Working;
            task_todo.start_timestamp = Some(NsTimestamp::now());
            task_todo.worker_pid = Some(self.pid);
            task_todo.worker_birth_timestamp = Some(self.birth_timestamp);
            // task_todo.comment = Some(String::from("this job was started by scoretracker-core"));

            let mut task = task_todo.clone();
            info!("taking on task with uuid: {}", task.uuid.0);

            queue.write_to_file().map_err(E::CannotWriteQueue)?;

            // Drop file lock here to and let other processes access the queue
            let queue = queue.close();

            // Do some task if there is something to do
            Self::execute_task(&mut task);

            // Update the queue file again to update the state of the task
            let mut queue = queue.reopen().map_err(E::CannotReopenQueue)?;
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
    fn execute_task(task: &mut Task) {
        log_fn_name!("worker:execute task");

        match task.job.run() {
            Ok(results) => {
                task.state = TaskState::Done;
                task.results = Some(results.into());
                success!("task finished successfully: uuid: {} results: {:#?}", task.uuid.0, results);
            }
            Err(error) => {
                task.state = TaskState::Failed;
                task.results = None; // todo - save errors
                error!("task failed: uuid: {} error: {:?}", task.uuid.0, error)
            }
        }
        task.finish_timestamp = Some(NsTimestamp::now());
    }

    /// Execute a task from the queue in the current thread.
    pub fn execute_task_by_uuid(&self, task_uuid: Uuid) -> Result<(), Error> {
        log_fn_name!("worker:execute_task_by_uuid");
        type E = Error;
        pub const QUEUE_PATH: &str = "test_queue.jsonl"; // todo

        // Read the queue to either add something or take on a task
        let mut queue = TaskQueue::read_or_create_new_safe(QUEUE_PATH).map_err(E::CannotReadQueue)?;

        // Take on a task
        let task_todo = queue.get_task_mut(task_uuid).ok_or(E::TaskNotFound(task_uuid))?;
        task_todo.state = TaskState::Working;
        task_todo.start_timestamp = Some(NsTimestamp::now());
        task_todo.worker_pid = Some(self.pid);
        task_todo.worker_birth_timestamp = Some(self.birth_timestamp);
        // task_todo.comment = Some(String::from("this job was started by scoretracker-core"));

        let mut task = task_todo.clone();
        info!("taking on task with uuid: {}", task.uuid.0);

        queue.write_to_file().map_err(E::CannotWriteQueue)?;

        // Drop file lock here to and let other processes access the queue
        let queue = queue.close();

        // Do some task if there is something to do
        Self::execute_task(&mut task);

        // Update the queue file again to update the state of the task
        let mut queue = queue.reopen().map_err(E::CannotReopenQueue)?;
        queue.update_task(task).map_err(E::CannotUpdateTask)?;
        queue.write_to_file().map_err(E::CannotWriteQueue)?;
        Ok(())
    }
}
