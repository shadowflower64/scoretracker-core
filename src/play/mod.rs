use crate::util::{timestamp::NsTimestamp, uuid::UuidString};
use serde::{Deserialize, Serialize};

// TODO
#[derive(Deserialize, Serialize)]
pub struct PlayDatabase {
    pub format_version: i32,
    pub plays: Vec<Box<dyn PlayTrait>>,
}

#[typetag::serde(tag = "type")]
pub trait PlayTrait {
    fn proof(&self) -> Vec<UuidString>;
    fn timestamp(&self) -> NsTimestamp;
    fn comment(&self) -> String;
}
