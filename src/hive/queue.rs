use crate::hive::task::{Task, TaskState};
use crate::util::file_ex::FileEx;
use crate::util::lockfile::{self, LockfileHandle};
use std::fmt;
use std::path::Path;
use uuid::Uuid;

pub struct TaskQueue {
    tasks: Vec<Task>,
    lockfile: LockfileHandle,
}

#[derive(Debug)]
pub struct TaskAlreadyExists;

impl fmt::Display for TaskAlreadyExists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "task with the same UUID already exists")
    }
}

impl std::error::Error for TaskAlreadyExists {}

#[derive(Debug)]
pub struct TaskNotFound;

impl fmt::Display for TaskNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "task with this UUID was not found")
    }
}

impl std::error::Error for TaskNotFound {}

impl TaskQueue {
    pub fn top_queued_task(&self) -> Option<&Task> {
        self.tasks.iter().find(|task| task.state == TaskState::Queued)
    }

    pub fn top_queued_task_mut(&mut self) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.state == TaskState::Queued)
    }

    /// Add a new task.
    ///
    /// This function adds a new task to the end of the queue.
    ///
    /// # Errors
    /// If the queue already has a task with the same UUID, nothing happens and an Err variant is returned.
    pub fn add_task(&mut self, task: Task) -> Result<(), TaskAlreadyExists> {
        if self.get_task(task.uuid.0).is_none() {
            self.tasks.push(task);
            Ok(())
        } else {
            Err(TaskAlreadyExists)
        }
    }

    /// Update an existing task.
    ///
    /// This function updates an existing task in the queue by finding the task with the same UUID and replacing it with the given task data.
    ///
    /// # Errors
    /// If the queue does not already have a task with the same UUID, nothing happens and an Err variant is returned.
    pub fn update_task(&mut self, task: Task) -> Result<(), TaskNotFound> {
        if let Some(old_task) = self.get_task_mut(task.uuid.0) {
            *old_task = task;
            Ok(())
        } else {
            Err(TaskNotFound)
        }
    }

    /// Add a new task or update an existing one.
    ///
    /// This function updates an existing task in the queue if a task with the same UUID exists already,
    /// and adds a new task if a task with this UUID is not present in the queue yet.
    ///
    /// This is like a combination of [`Self::add_task`] and [`Self::update_task`].
    pub fn add_or_update_task(&mut self, task: Task) {
        if let Some(old_task) = self.get_task_mut(task.uuid.0) {
            *old_task = task;
        } else {
            self.tasks.push(task);
        }
    }

    pub fn get_task(&self, task_uuid: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|task| task.uuid.0 == task_uuid)
    }

    pub fn get_task_mut(&mut self, task_uuid: Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.uuid.0 == task_uuid)
    }

    pub fn read_or_create_new_safe<P: AsRef<Path>>(path: P) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path)?;
        let tasks = lockfile.read_from_jsonlines()?.unwrap_or_default();
        Ok(Self { tasks, lockfile })
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_jsonlines(&self.tasks)?)
    }
}
