//! Library cache file handling.
//!
//! A library cache is a local cache file that collects file names, file sizes and file timestamps from a library directory,
//! and maps them to a SHA256 hash, to prevent repeated expensive hash calculations for unchanged files.
//!
//! While a library cache does not contain any important data and can be safely removed, doing so will significantly increase the
//! next library scan duration. Scanning a 3,000-file library without a cache may take several hours to complete.
//! Therefore, this file should not be deleted often.
use crate::library::index::LibraryIndex;
use crate::util::file_ex::{self, FileEx};
use crate::util::timestamp::NsTimestamp;
use crate::{debug, log_fn_name, log_should_print_debug};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Library cache entry for one file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileCacheInfo {
    /// Name of the file. Non-UTF-8 sequences are replaced with [`std::char::REPLACEMENT_CHARACTER`] using [`std::ffi::OsStr::to_string_lossy()`]
    pub filename: String,

    /// Size of the file (in bytes).
    pub file_size: u64,

    /// Birth timestamp of the file.
    pub birth_timestamp: NsTimestamp,

    /// Modification timestamp of the file.
    pub modify_timestamp: NsTimestamp,

    /// SHA256 hash in a hexadecimal string format.
    pub sha256: String,
}

/// Inner structure for [`LibraryCache`].
///
/// This structure contains data that is actually deserialized/serialized into the cache file.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryCache {
    pub files: Vec<FileCacheInfo>,
}

/// Cache containing hashes of library files.
///
/// The library cache is a file used to avoid repeated hash calculations for file contents.
///
/// If a file has the exact same filename, the exact same file size, the exact same birth timestamp and modification timestamp (with nanosecond precision)
/// to a file recorded in the cache, it can be assumed to have the same SHA256 hash to the file recorded in the cache.
///
/// If any of these values are not identical to an entry in the cache, the file can be assumed to be different, and the hash can be recalculated.
/// The newly calculated hash can also be added to the cache for future use.
///
/// This is a wrapper structure for [`LibraryCache`]. Apart from the data, it also contains the file path of the cache file.
#[derive(Debug, Clone)]
pub struct LibraryCacheLock {
    inner: LibraryCache,
    file_path: PathBuf,
}

impl LibraryCacheLock {
    /// Determines whether or not the cache data should be automatically saved to disk after adding a new entry.
    ///
    /// This is recommended to be `true` as it reduces the risk of having to do all of the hash calculations all over when the program crashes.
    const AUTOSAVE: bool = true;

    /// Determines whether the JSON written to file should contain unnecessary whitespace or not.
    ///
    /// This is recommended to be `false` as it reduces the final written file size and speeds up the saving process, especially when autosaving.
    pub const WRITE_PRETTY_JSON: bool = false;

    /// Standard filename used for library cache
    pub const STANDARD_FILENAME: &str = "library_cache.json";

    /// Function for filtering the cache to find a specific entry.
    ///
    /// This function takes some parameters and returns a predicate function that can filter the cache for specific results.
    /// This function can operate over immutable references to [`FileCacheInfo`].
    fn cache_find_predicate(
        filename: &str,
        file_size: u64,
        birth_timestamp: NsTimestamp,
        modify_timestamp: NsTimestamp,
    ) -> impl Fn(&&FileCacheInfo) -> bool {
        move |cache_entry| {
            cache_entry.file_size == file_size
                && cache_entry.birth_timestamp == birth_timestamp
                && cache_entry.modify_timestamp == modify_timestamp
                && cache_entry.filename == filename
        }
    }

    /// Function for filtering the cache to find a specific entry.
    ///
    /// This function takes some parameters and returns a predicate function that can filter the cache for specific results.
    /// This function can operate over mutable references to [`FileCacheInfo`].
    ///
    /// The returned predicate transforms the mutable reference into an immutable reference and reuses [`Self::cache_find_predicate`] to provide the same functionality.
    fn cache_find_predicate_mut(
        filename: &str,
        file_size: u64,
        birth_timestamp: NsTimestamp,
        modify_timestamp: NsTimestamp,
    ) -> impl Fn(&&mut FileCacheInfo) -> bool {
        move |x| {
            let y = &&**x;
            Self::cache_find_predicate(filename, file_size, birth_timestamp, modify_timestamp)(y)
        }
    }

    /// Finds the cached SHA256 hash of a file given the file's identifying characteristics.
    ///
    /// This function takes the name of the file, the size of the file, as well as the file's birth timestamp and modification timestamp,
    /// and returns a cached SHA256 hash of an entry that has all of these values identical to the provided ones.
    ///
    /// This function returns [`None`] if no cached entry matches the given parameters.
    pub fn find_cached_sha256_hash(
        &self,
        filename: &str,
        file_size: u64,
        birth_timestamp: NsTimestamp,
        modify_timestamp: NsTimestamp,
    ) -> Option<String> {
        self.inner
            .files
            .iter()
            .find(Self::cache_find_predicate(filename, file_size, birth_timestamp, modify_timestamp))
            .map(|cache_entry| cache_entry.sha256.clone())
    }

    /// Finds the cached SHA256 hash of a file, or computes the hash if it is not cached yet.
    ///
    /// This function takes in a path of the file, and returns a SHA256 hash. The function uses the cache to avoid doing repeated calculations.
    ///
    /// If this file has not been recorded in the cache yet, this function will read in the whole file,
    /// compute the hash of the file, update the cache file and save it to disk automatically.
    pub fn find_or_compute_file_sha256_hash(&mut self, path: &Path) -> String {
        log_fn_name!("scan:find_or_compute_file_sha256_hash");
        log_should_print_debug!(LibraryIndex::VERBOSE_SCANNING);

        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_size = fs::metadata(path).unwrap().size();
        let birth_timestamp = fs::metadata(path).unwrap().created().unwrap().into();
        let modify_timestamp = fs::metadata(path).unwrap().modified().unwrap().into();

        if let Some(cached_hash) = self.find_cached_sha256_hash(&filename, file_size, birth_timestamp, modify_timestamp) {
            debug!("using cached hash for {path:?}: {cached_hash}");
            cached_hash
        } else {
            let computed_hash = compute_hash_of_file(path);
            self.insert(filename, file_size, birth_timestamp, modify_timestamp, computed_hash.clone());

            if Self::AUTOSAVE {
                self.write_to_file().expect("could not autosave cache to file");
            }

            computed_hash
        }
    }

    /// Inserts a new cache entry or updates an existing one.
    ///
    /// This function takes in identifiers of a file and the SHA256 hash of that file, and either adds a new cache entry,
    /// or updates an existing one if the identifiers match up with an existing cache entry.
    pub fn insert(
        &mut self,
        filename: String,
        file_size: u64,
        birth_timestamp: NsTimestamp,
        modify_timestamp: NsTimestamp,
        sha256: String,
    ) {
        if let Some(existing) = self.inner.files.iter_mut().find(Self::cache_find_predicate_mut(
            &filename,
            file_size,
            birth_timestamp,
            modify_timestamp,
        )) {
            existing.filename = filename;
            existing.file_size = file_size;
            existing.birth_timestamp = birth_timestamp;
            existing.modify_timestamp = modify_timestamp;
            existing.sha256 = sha256;
        } else {
            self.inner.files.push(FileCacheInfo {
                filename,
                birth_timestamp,
                modify_timestamp,
                file_size,
                sha256,
            });
        }
    }

    /// Loads cache data from a file or creates a new cache.
    ///
    /// This function loads the cache from a JSON file at the provided file path, or creates a new cache structure if the file does not exist.
    ///
    /// This function will return Err when:
    /// * the file could not be read to string, or
    /// * the JSON structure could not be parsed.
    ///
    /// This is to prevent overwriting existing data if it has become corrupted or protected by permissions.
    pub fn read_or_create_new(cache_file_path: PathBuf) -> file_ex::Result<Self> {
        log_fn_name!("scan:read_or_create_new");
        log_should_print_debug!(LibraryIndex::VERBOSE_SCANNING);

        let inner_opt = cache_file_path.read_from_json()?;
        if inner_opt.is_some() {
            debug!("loading library cache from {cache_file_path:?}");
        } else {
            debug!("creating library cache at {cache_file_path:?}");
        }

        let inner = inner_opt.unwrap_or_default();
        Ok(Self {
            inner,
            file_path: cache_file_path,
        })
    }

    /// Saves the cache file to disk.
    ///
    /// This function uses the stored file path to save the cache data to file.
    /// Depending on the constant [`Self::WRITE_PRETTY_JSON`], this function either writes the data using [`serde_json::to_string`] or [`serde_json::to_string_pretty`].
    pub fn write_to_file(&self) -> file_ex::Result<()> {
        let _ = self.file_path.parent().and_then(|parent| fs::create_dir_all(parent).ok());
        if Self::WRITE_PRETTY_JSON {
            self.file_path.write_as_json_pretty(&self.inner)?;
        } else {
            self.file_path.write_as_json(&self.inner)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum HashingMethod {
    SHA256,
    #[allow(unused)]
    MD5,
}

pub fn compute_hash_of_file(path: &Path) -> String {
    log_fn_name!("scan:compute_hash_of_file");
    log_should_print_debug!(LibraryIndex::VERBOSE_SCANNING);

    // note: changing the hashing method does not change the behaviour/naming of the hash in other places - everywhere else its still called sha256
    const METHOD: HashingMethod = HashingMethod::SHA256;
    debug!("computing hash for {path:?} using {METHOD:?}...");

    let bytes = std::fs::read(path).unwrap();
    let hash = match METHOD {
        HashingMethod::SHA256 => sha256::digest(&bytes),
        HashingMethod::MD5 => {
            format!("{:x}", md5::compute(&bytes))
        }
    };

    debug!("computing hash for {path:?} using {METHOD:?}... done: {hash}");
    hash
}
