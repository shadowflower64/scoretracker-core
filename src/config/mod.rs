use crate::hive::worker::WorkerInfo;
use crate::library::aux_data::LibraryAuxDataLock;
use crate::library::cache::LibraryCacheLock;
use crate::library::database::LibraryDatabaseLock;
use crate::library::index::LibraryIndex;
use crate::util::dirs::config_dir;
use crate::util::file_ex::FileEx;
use crate::util::lockfile::{self, LockfileHandle};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    pub domain_name: String,
    pub shared_data_repo_path: PathBuf,
    pub default_library_dir_path: PathBuf,
}

impl Config {
    pub fn library_database_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(LibraryDatabaseLock::STANDARD_FILENAME)
    }

    pub fn default_library_path(&self) -> PathBuf {
        self.default_library_dir_path.clone()
    }

    pub fn default_library_index_path(&self) -> PathBuf {
        self.default_library_dir_path.join(LibraryIndex::STANDARD_FILENAME)
    }

    pub fn default_library_cache_path(&self) -> PathBuf {
        self.default_library_dir_path.join(LibraryCacheLock::STANDARD_FILENAME)
    }

    pub fn default_library_aux_data_path(&self) -> PathBuf {
        self.default_library_dir_path.join(LibraryAuxDataLock::STANDARD_FILENAME)
    }
}

#[derive(Debug)]
pub struct ConfigLock {
    pub inner: Config,
    lockfile: LockfileHandle,
}

impl ConfigLock {
    pub const STANDARD_FILENAME: &str = "scoretracker_config.json";

    pub fn default_path() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }

    pub fn read_or_create_new_safe<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = lockfile.read_from_json()?.unwrap_or_default();
        Ok(Self { inner, lockfile })
    }

    pub fn read_or_create_new_default_safe(worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        Self::read_or_create_new_safe(Self::default_path(), worker_info)
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_json_pretty(&self.inner)?)
    }
}
