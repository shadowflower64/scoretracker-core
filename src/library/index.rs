use crate::library::{cache::LibraryCacheLock, database::LibraryDatabaseLock};
use crate::util::file_ex::{Error, FileEx};
use crate::util::uuid::UuidString;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;
use std::{collections::HashMap, path::Path};
use walkdir::WalkDir;

/// A mapping from paths to proof UUIDs.
///
/// The library index is a data structure that links specific proof files on disk to proof UUIDs.
///
/// The index does not actually store any meaningful data, and is used only as a quick-access list of all available proof files in the library.
///
/// # Usage
///
/// Here's an example use case of the index:
/// Let's say the user wants to list all available files in the proof library and view the associated scores.
/// Without the index file, the software would have to:
/// 1. Recursively search through the entire library.
/// 2. Calculate the SHA256 hashes of found files (or fetch them from [`LibraryCache`]).
/// 3. Open the proof database.
/// 4. Search through all proofs and filter for the ones with matching SHA256 hashes.
/// 5. Note the UUIDs of the filtered proofs.
/// 6. Open the play database.
/// 7. Search through all scores that reference the proof UUIDs found before.
///
/// With the index file, finding the proof's UUID is as easy as a hashmap lookup. With an up-to-date index file, the process looks like this:
/// 1. Open the index file.
/// 2. Retrieve all proof UUIDs and paths from the index directly.
/// 3. Open the play database.
/// 4. Search through all scores that reference the proof UUIDs found before.
///
/// Searching through the entire directory recurisvely is moved to the scanning process which can be launched separately,
/// and will only need to be ran once new files are added to the library. See below for details.
///
/// # Scanning
///
/// This data structure goes out of date whenever a new proof file gets added to the library,
/// whenever a proof file gets moved around the library, and whenever a proof is removed from the library.
/// To sync up the data structure again, the library needs to be *rescanned*.
/// Scanning can be done via the [`LibraryIndex::scan_library_dir`] function, which returns
/// an entirely new index structure, which can then be saved to disk.
/// The saved file is usually called [`LibraryIndex::STANDARD_FILENAME`].
#[derive(Debug, Clone, Serialize, Default)]
pub struct LibraryIndex {
    // Map of 'relative file path' : 'proof UUID'
    pub files: HashMap<PathBuf, UuidString>,
}

impl LibraryIndex {
    pub const VERBOSE_SCANNING: bool = true;
    pub const STANDARD_FILENAME: &str = "library_index.json";

    pub fn should_file_be_scanned(filename: &str) -> bool {
        filename.ends_with(".mp4") || filename.ends_with(".mkv")
    }

    pub fn scan_library_dir(library_dir: &Path, library_data: &mut LibraryDatabaseLock) -> Self {
        let scanning_start_timestamp = Instant::now();

        let mut index = Self::default();
        let mut cache = LibraryCacheLock::read_or_create_new(library_dir.join(LibraryCacheLock::STANDARD_FILENAME))
            .expect("could not read library cache");

        let files_to_scan: Vec<_> = WalkDir::new(library_dir)
            .into_iter()
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|dir_entry| if dir_entry.file_type().is_file() { Some(dir_entry) } else { None })
            })
            .collect();
        let len = files_to_scan.len();
        let mut skipped = 0;

        for (i, dir_entry) in files_to_scan.iter().enumerate() {
            let path = dir_entry.path();

            let is_supposed_to_be_scanned = Self::should_file_be_scanned(dir_entry.file_name().to_os_string().to_string_lossy().as_ref());
            if !is_supposed_to_be_scanned {
                if LibraryIndex::VERBOSE_SCANNING {
                    println!("[scan] [{i}/{len}] skipping {path:?}");
                }
                skipped += 1;
                continue;
            }

            if LibraryIndex::VERBOSE_SCANNING {
                println!("[scan] [{i}/{len}] scanning {path:?}");
            }

            let sha256_hash = cache.find_or_compute_file_sha256_hash(path);
            let uuid = if let Some(entry) = library_data.find_entry_by_sha256_hash(&sha256_hash) {
                let uuid = entry.uuid.0;
                if LibraryIndex::VERBOSE_SCANNING {
                    println!("[scan] found duplicate file: sha256: {sha256_hash}, uuid: {uuid}");
                }
                // TODO: record this duplicate file path in the library entry
                uuid
            } else {
                library_data.add(path, sha256_hash)
            };
            index.files.insert(path.to_owned(), uuid.into());
        }

        let scanning_end_timestamp = Instant::now();
        let scanning_duration = scanning_end_timestamp.duration_since(scanning_start_timestamp);

        println!(
            "scanning done; took {scanning_duration:?}; {len} files found: {} files scanned in, {skipped} files skipped",
            len - skipped
        );

        index
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        path.write_as_json_pretty(self)?;
        Ok(())
    }
}
