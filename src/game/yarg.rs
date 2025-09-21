use crate::play::PlayTrait;
use crate::util::timestamp::NsTimestamp;
use crate::{game::yarg, util::uuid::UuidString};
use serde::{Deserialize, Serialize};

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

/// Difficulty that the play was performed on.
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

/// A YARG play - a performance of one player playing on one instrument on a specific chart.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Play {
    pub song_id: String,
    pub instrument: Instrument,
    pub difficulty: Difficulty,
    pub mode: Mode,
    pub score: u64,
    pub notes_hit: u64,
    pub max_streak: u64,
    pub overhits: u64,
    pub song_speed: f64,
    pub modifiers: Vec<Modifier>,
    pub game_version: String,
    pub proof: Vec<UuidString>,
    pub timestamp: NsTimestamp,
    pub comment: String,
}

#[typetag::serde]
impl PlayTrait for yarg::Play {
    fn proof(&self) -> Vec<UuidString> {
        self.proof.clone()
    }
    fn timestamp(&self) -> NsTimestamp {
        self.timestamp
    }
    fn comment(&self) -> String {
        self.comment.clone()
    }
}
