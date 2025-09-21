use crate::util::file_ex::FileEx;
use crate::util::lockfile::{self, LockfileHandle};
use crate::util::timestamp::NsTimestamp;
use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use uuid::Uuid;

/// Basic metadata about the file from the `stat` command.
///
/// This struct stores basic metadata about the file, such as the file's size, the file modification time, and the file creation time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileStat {
    /// Size of the file, in bytes.
    pub size: u64,

    /// Birth of the file - when was this file created on the disk?
    ///
    /// For raw video files, this is usually the time when the video has started recording.
    pub timestamp_birth: NsTimestamp,

    /// Access of the file - when was this file last accessed or read?
    pub timestamp_access: NsTimestamp,

    /// Modification - when was the data inside of this file modified? For raw video files, this is usually the time when the video has finished recording.
    ///
    /// This value may be set by tools such as LosslessCut to indicate a video recording timestamp, however it may be wrong.
    /// I think LosslessCut actually moves the timestamp wrongly.
    pub timestamp_modification: NsTimestamp,

    /// Status change - when were the permissions(?) changed for this file?
    pub timestamp_status_change: NsTimestamp,

    /// TimestaWhen was the file stat read? (This is not actually part of the `stat` command, and it is stored manually.)
    pub last_check: NsTimestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MediaCategory {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// An image of the screen captured from a PC.
    PCScreenshot,

    /// An image of the screen captured from a phone.
    MobileScreenshot,

    /// An image captured by a photo camera, a phone camera, or a webcam.
    CameraPhoto,

    /// A video of the screen captured by OBS Studio.
    ObsRecording,

    /// A video of the screen captured by OBS Studio, and then cut using the `autocut` script.
    ObsRecordingAutocut,

    /// A video of the screen captured by OBS Studio, and then cut using LosslessCut.
    ObsRecordingLosslessCut,

    /// A video of the screen captured by a phone's screen recording software.
    MobileScreenRecording,

    /// A video captured by a photo camera, a phone camera, or a webcam.
    CameraVideo,

    /// Other media, that doesn't belong to any other category.
    Other,
}

pub type GameId = String;
pub type Tag = String;

/// The contents of the video or image that the library entry is associated with - what kind of footage does the video show?
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", tag = "category")]
pub enum ContentDescription {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// The video shows the song select screen, the entire playthrough of one song, and the end screen.
    GameplayNormal { game: Option<GameId> },

    /// The video or image shows only gameplay, and does not show the score screen at the end.
    GameplayOnly { game: Option<GameId> },

    /// The video or image shows only the results screen, and does not show the gameplay.
    ResultsScreen { game: Option<GameId> },

    /// The video or image depicts some part of the game, but the contents of the video or image don't belong to any other more specific category.
    GameGeneric { game: Option<GameId> },

    /// The contents of the video or image don't belong to any other category.
    Other,
}

/// The quality state of the proof file.
///
/// Videos that are "raw" can be transcoded and lossily compressed to save space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QualityState {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// Raw unprocessed recording or replay file or stream vod, which may or may not have been cut using `ffmpeg` with with `-c copy`, or LosslessCut. Largest file and best quality.
    /// Not recommended to store for a long time.
    Raw,

    /// Transcoded cut video, but visually lossless. Takes up a lot less space because it is transcoded after the initial recording on a slower encoding preset.
    /// Useful for PBs and first FCs.
    Compressed,

    /// Transcoded cut video, in 720p but still readable quality. Has to take up less than 10 MiB per 2.5 minutes of video.
    /// Useful for non-PB plays that would've usually been thrown in the trash entirely.
    Crumpled,

    /// Transcoded cut video, with terrible bitrate and 360p. Takes up around 1-3 MiB per 2.5 minutes of video.
    /// Useful for unfinished plays or otherwise something that should be deleted usually, but may come in handy later (for example, for counting attempts).
    Shredded,
}

/// Kind of the library entry - is it a proof of a play or something else?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LibraryEntryKind {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// Video not showing a play, unrelated to proof stuff but still in library for some reason.
    Unrelated,

    /// Video showing a play, but not yet possible to associate with a play - the play is not saveable in database for some reason. for example, one-finger-challenge FCs.
    Unsupported,

    /// Video showing a play, but not yet associated with a play.
    NotLinkedYet,

    /// Video showing a play, associated with a play or multiple plays.
    Linked,
}

pub type MediaMetadata = HashMap<String, String>;

/// An entry in the library database, containing information about proof videos and images, and other files inside of the library.
///
/// Every unique file inside of the library should have exactly one library entry.
/// Old files, which have been deleted, moved, or transcoded into other files, should not have their entries removed from the library.
/// This is to preserve information about the source files for processed and cut files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    /// UUID of the library entry / proof.
    pub uuid: UuidString,

    /// SHA256 hash of the file.
    pub sha256: String,

    /// Known library locations of the file. Updated on rescan.
    pub library_urls: Vec<String>,

    /// Is the media file linked to any play? Will it be linked to a play in the future? Or is this not a video of a play at all?
    pub entry_kind: LibraryEntryKind,

    /// Some information about the file from `stat`.
    pub file_stat: Option<FileStat>,

    /// Metadata inside of the media file (creation_date, android version, video/audio stream count, other similar metadata).
    /// The exact contents depends on the type of the file.
    ///
    /// Currently, this is not used, and the metadata will always be empty.
    pub metadata: Option<MediaMetadata>,

    /// Category of the media that this entry describes - is it a screenshot, a video from a camera, a mobile screen recording, something else?
    pub media_category: MediaCategory,

    /// Content of the video - whether the video is showing gameplay, just the results, or something else. This field also contains information about the game being played.
    ///
    /// This field can be used by sorting and filtering systems to show relevant videos to the user.
    pub content_description: ContentDescription,

    /// Is this a full raw recording/stream vod, or is it cut already and shows only the relevant play?
    ///
    /// Set this to [`None`] if it is unknown whether the video has been cut or not.
    pub cut: Option<bool>,

    /// Is the video raw, compressed, crumpled, or shredded?
    pub quality: QualityState,

    /// An entry UUID of the source media file that this file was cut out from. Files cut out from the same file are said to be "cut from the same cloth".
    ///
    /// Set this to [`None`] if the cloth is not known, or the file is not cut.
    pub cloth: Option<UuidString>,

    /// An entry UUID of the source media file that this file was processed from. Pre-processed files are "dry" and post-processed files are "wet".
    ///
    /// Set this to [`None`] if the dry file is not known, or the file is not processed.
    pub dry: Option<UuidString>,

    /// List of entry UUIDs of source media files used to create this media file. Montages are made of multiple clips for example.
    ///
    /// Set this to `Some(Vec::new())` if the clips are not known. Set this to [`None`] if this is not a montage.
    pub clips: Option<Vec<UuidString>>,

    /// List of tags that are assigned to this library entry by the user.
    pub tags: HashSet<Tag>,

    /// User-added comment for this library entry.
    pub comment: Option<String>,

    /// Timestamp (in nanoseconds) of when this file was added/scanned into the library.
    pub timestamp_added: NsTimestamp,
}

impl Default for LibraryEntry {
    fn default() -> Self {
        Self {
            // Explicitly set custom values
            uuid: Uuid::new_v4().into(),
            timestamp_added: NsTimestamp::now(),

            // Default values for other fields
            sha256: String::new(),
            library_urls: Vec::new(),
            entry_kind: LibraryEntryKind::default(),
            file_stat: None,
            metadata: None,
            media_category: MediaCategory::default(),
            content_description: ContentDescription::default(),
            cut: None,
            quality: QualityState::default(),
            cloth: None,
            dry: None,
            clips: None,
            tags: HashSet::new(),
            comment: None,
        }
    }
}

#[derive(Debug)]
pub struct LibraryDatabase {
    entries: Vec<LibraryEntry>,
    lockfile: LockfileHandle,
}

impl LibraryDatabase {
    pub const STANDARD_FILENAME: &str = "library_database.json";

    pub fn find_entry_by_sha256_hash(&self, sha256: &str) -> Option<&LibraryEntry> {
        self.entries.iter().find(|x| x.sha256 == sha256)
    }

    pub fn add(&mut self, file_path: &Path, sha256: String) -> Uuid {
        const DOMAIN: &str = "domain.example.com"; // TODO
        let relative_file_path = file_path.to_string_lossy().to_string(); // TODO
        let library_entry = LibraryEntry {
            library_urls: vec![format!("stpl://{DOMAIN}/{relative_file_path}")],
            sha256,
            ..Default::default()
        };
        let uuid = library_entry.uuid.0;
        self.entries.push(library_entry);
        uuid
    }

    pub fn read_or_create_new_safe<P: AsRef<Path>>(path: P) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path)?;
        let entries = lockfile.read_from_jsonlines()?.unwrap_or_default();
        Ok(Self { entries, lockfile })
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_jsonlines(&self.entries)?)
    }
}
