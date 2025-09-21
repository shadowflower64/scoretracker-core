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
pub struct LibraryAuxDataInner {
    pub tags: Vec<TagInfo>,
}

/// Wrapper for handling auxiliary library data files. See [`LibraryAuxDataInner`] for more documentation.
#[derive(Debug)]
pub struct LibraryAuxData {
    inner: LibraryAuxDataInner,
    lockfile: LockfileHandle,
}

impl LibraryAuxData {
    pub fn read_or_create_new_safe<P: AsRef<Path>>(path: P) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path)?;
        let inner = lockfile.read_from_json()?.unwrap_or_default();
        Ok(Self { inner, lockfile })
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_json_pretty(&self.inner)?)
    }
}
