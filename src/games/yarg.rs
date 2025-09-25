//! Data structures for YARG (Yet Another Rhythm Game).
use std::collections::HashMap;

use crate::game::Game;
use crate::scoreboard::performance::{self, PerformanceMetadata};
use crate::songdb::song::{self, SongAlbumInfo};
use crate::util::cmd::{AskError, ask};
use crate::util::percentage::Percentage;
use crate::util::timestamp::NsTimestamp;
use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A playable part in the chart.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    LeadGuitar,
    MelodyGuitar,
    RhythmGuitar,
    BassGuitar,
    Drums4L,
    Drums5L,
    ProDrums,
    EliteDrums,
    Keys5L,
    ProKeys,
    Vocals,
    Harmony1,
    Harmony2,
    Harmony3,
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Beginner,
    Easy,
    Medium,
    Hard,
    Expert,
    ExpertPlus,
}

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Quickplay,
}

/// A modifier (chart mutator).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    AllStrums,
    AllHopos,
    AllTaps,
    HoposToTaps,
    TapsToHopos,
    NoRangeShifts,
    NoKicks,
    NoDynamics,
}

/// A YARG performance - a performance of one player playing on one instrument on a specific chart.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    /// Player UUID.
    pub player_uuid: UuidString,

    /// Named ID of the song.
    pub song_id: String,

    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Game mode that this performance was played on.
    pub mode: Mode,

    /// Amount of points at the end of the performance.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u64,

    /// The maximum streak achieved during the performance.
    pub max_streak: u64,

    /// The amount of extra erroneous inputs.
    pub overhits: u64,

    /// Speed of the song, as a percentage. This is not a normal `f64` to avoid rounding errors.
    pub song_speed: Percentage,

    /// List of modifiers that were used during this performance.
    pub modifiers: Vec<Modifier>,

    /// String of the game version that was played on for this performance.
    pub game_version: String,

    /// List of library entry UUIDs that are proof of this performance.
    pub proof: Vec<UuidString>,

    /// Timestamp of the performance - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    pub timestamp: NsTimestamp,

    /// Optional user comment.
    pub comment: Option<String>,

    /// Any additional performance metadata.
    pub metadata: PerformanceMetadata,
}

#[typetag::serde(name = "yarg")]
impl performance::Performance for Performance {
    fn proof(&self) -> Vec<UuidString> {
        self.proof.clone()
    }
    fn timestamp(&self) -> NsTimestamp {
        self.timestamp
    }
    fn comment(&self) -> Option<String> {
        self.comment.clone()
    }
    fn metadata(&self) -> PerformanceMetadata {
        self.metadata.clone()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: SongAlbumInfo,
    pub year: String,
}

#[typetag::serde(name = "yarg")]
impl song::Song for Song {
    fn title(&self) -> String {
        self.title.clone()
    }
    fn artist(&self) -> String {
        self.artist.clone()
    }
    fn album(&self) -> Option<SongAlbumInfo> {
        Some(self.album.clone())
    }
    fn year(&self) -> Option<i64> {
        self.year.parse().ok()
    }
}

#[derive(Deserialize, Serialize)]
pub struct YARG {}

#[typetag::serde(name = "yarg")]
impl Game for YARG {
    fn pretty_name(&self) -> &'static str {
        "Yet Another Rhythm Game"
    }

    fn ask_for_performance(&self) -> Result<Box<dyn performance::Performance>, AskError> {
        Ok(Box::new(Performance {
            player_uuid: Uuid::parse_str(&ask("player uuid")?).unwrap().into(),
            song_id: ask("song id")?,
            instrument: Instrument::LeadGuitar,
            difficulty: Difficulty::Expert,
            mode: Mode::Quickplay,
            score: 0,
            notes_hit: 0,
            max_streak: 0,
            overhits: 0,
            song_speed: Percentage(100.0),
            modifiers: Vec::new(),
            game_version: String::new(),
            proof: Vec::new(),
            timestamp: NsTimestamp::now(),
            comment: None,
            metadata: HashMap::new(),
        }))
    }
}
