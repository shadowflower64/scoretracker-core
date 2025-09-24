use crate::hive::worker::WorkerInfo;
use crate::library::aux_data::LibraryAuxDataLock;
use crate::library::cache::LibraryCacheLock;
use crate::library::database::LibraryDatabaseLock;
use crate::library::index::LibraryIndex;
use crate::util::dirs::config_dir;
use crate::util::file_ex::{self, FileEx};
use crate::util::lockfile::{self, LockfileHandle};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub domain_name: String,
    pub shared_data_repo_path: PathBuf,
    pub default_library_dir_path: PathBuf,
}

impl Config {
    pub fn load() -> lockfile::Result<Self> {
        Self::load_with_worker(None)
    }

    pub fn load_with_worker(worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        ConfigLock::read_default_safe(worker_info).map(|lock| lock.inner)
    }

    pub fn library_database_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(LibraryDatabaseLock::STANDARD_FILENAME)
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

    pub fn read_safe<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = lockfile.read_from_json()?.ok_or(file_ex::Error::file_not_found())?;
        Ok(Self { inner, lockfile })
    }

    pub fn read_default_safe(worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        let config_path = env::var("SCORETRACKER_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or(Self::default_path());
        Self::read_safe(config_path, worker_info)
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_json_pretty(&self.inner)?)
    }
}
