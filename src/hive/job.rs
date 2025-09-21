use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf, thread::sleep, time::Duration};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingType {
    CompressImportantVideo,
    CompressCrumpleVideo,
    CompressShredVideo,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
#[serde(rename_all = "snake_case")]
pub enum Job {
    Sleep {
        time_nanos: i128,
    },
    DisplayMessage {
        message: String,
    },
    DisplayMessageAndSleep {
        message: String,
        time_nanos: i128,
    },
    CutVideo {
        source_proof_uuid: UuidString,
        source_path: PathBuf,
        cut_point_start_ms: Option<u64>,
        cut_point_end_ms: Option<u64>,
        destination_path: PathBuf,
    },
    ProcessVideo {
        source_proof_uuid: UuidString,
        source_path: PathBuf,
        processing_type: ProcessingType,
        destination_path: PathBuf,
    },
}

#[derive(Debug)]
pub struct Error {}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
        // write!(f, "unknown error while running a job") // todo
    }
}

impl std::error::Error for Error {}

impl Job {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Job::DisplayMessage { message } => println!("{}", message),
            Job::Sleep { time_nanos } => sleep(Duration::from_nanos(*time_nanos as u64)),
            Job::DisplayMessageAndSleep { message, time_nanos } => {
                println!("{}", message);
                sleep(Duration::from_nanos(*time_nanos as u64));
            }
            Job::CutVideo { .. } => todo!(),
            Job::ProcessVideo { .. } => todo!(),
        }
        Ok(())
    }
}
