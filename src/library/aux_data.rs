use crate::hive::worker::WorkerInfo;
use crate::util::file_ex::FileEx;
use crate::util::lockfile::{self, LockfileHandle};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TagInfo {
    pub id: String,
    pub name: String,
    pub color: (), // TODO
}

/// Auxiliary data for the library.
///
/// This structure contains information on various additional aspects of the library, that are not proof entries.
/// For example, it contains information about the names and colors of tags used in the library.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryAuxData {
    pub tags: Vec<TagInfo>,
}

/// Wrapper for handling auxiliary library data files. See [`LibraryAuxData`] for more documentation.
#[derive(Debug)]
pub struct LibraryAuxDataLock {
    inner: LibraryAuxData,
    lockfile: LockfileHandle,
}

impl LibraryAuxDataLock {
    pub const STANDARD_FILENAME: &str = "library_aux.json";

    pub fn read_or_create_new_safe<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = lockfile.read_from_json()?.unwrap_or_default();
        Ok(Self { inner, lockfile })
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_json_pretty(&self.inner)?)
    }
}
