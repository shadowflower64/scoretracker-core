/// Library cache file handling.
///
/// A library cache is a local cache file that collects file names, file sizes and file timestamps from a library directory,
/// and maps them to a SHA256 hash, to prevent repeated expensive hash calculations for unchanged files.
///
/// While a library cache does not contain any important data and can be safely removed, doing so will significantly increase the
/// next library scan duration. Scanning a 3,000-file library without a cache may take several hours to complete.
/// Therefore, this file should not be deleted often.
pub mod cache;

/// Library database file handling.
///
/// A library database file is a file shared globally across libraries, that maps "proof UUIDs" to actual information and metadata about the proof.
/// Every entry in a library database file contains information about the SHA256 hash of the proof file, the type of the file (recording, screenshot etc.),
/// the modification timestamps of the file, the state of the file (is it linked to any score? is it uploaded), as well as other information.
pub mod database;

/// Library auxililary data file handling.
///
/// A "library auxiliary data file" is a file that contains additional information about the library that is not the actual files of the library.
/// For example, auxiliary data may contain information about the library's tags.
pub mod aux_data;

/// Library index file handling.
///
/// A library index is a file that maps every filename existing in the library directory to a "proof UUID", which should be shared across all connected libraries.
///
/// This file is re-created every time the library gets re-scanned for new content.
pub mod index;
