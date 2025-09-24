use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::path::Path;
use std::{io, result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot read from file: {0}")]
    CannotReadFile(io::Error),
    #[error("cannot write to file: {0}")]
    CannotWriteFile(io::Error),
    #[error("cannot deserialize json: {0}")]
    CannotDeserializeJSON(serde_json::Error),
    #[error("cannot serialize json: {0}")]
    CannotSerializeJSON(serde_json::Error),
    #[error("cannot deserialize jsonlines: {0}")]
    CannotDeserializeJSONLines(io::Error),
    #[error("cannot serialize jsonlines: {0}")]
    CannotSerializeJSONLines(io::Error),
}

impl Error {
    pub fn file_not_found() -> Self {
        Self::CannotReadFile(io::Error::from(io::ErrorKind::NotFound))
    }
}

pub type Result<T> = result::Result<T, Error>;

pub trait FileEx {
    fn file_path(&self) -> &Path;

    fn read_to_string(&self) -> io::Result<Option<String>> {
        match fs::read_to_string(self.file_path()) {
            Ok(content) => Ok(Some(content)),
            Err(e) => {
                if matches!(e.kind(), io::ErrorKind::NotFound) {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    fn read_from_json<D: for<'a> Deserialize<'a>>(&self) -> Result<Option<D>> {
        let content = self.read_to_string().map_err(Error::CannotReadFile)?;
        if let Some(json) = content {
            let deserialized = serde_json::from_str(&json).map_err(Error::CannotDeserializeJSON)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    fn read_from_jsonlines<D: for<'a> Deserialize<'a>>(&self) -> Result<Option<Vec<D>>> {
        let result = serde_jsonlines::json_lines(self.file_path());
        match result {
            Err(e) => {
                if matches!(e.kind(), io::ErrorKind::NotFound) {
                    Ok(None)
                } else {
                    Err(Error::CannotReadFile(e))
                }
            }
            Ok(iter) => Ok(Some(
                iter.collect::<io::Result<Vec<D>>>().map_err(Error::CannotDeserializeJSONLines)?,
            )),
        }
    }

    fn write<C: AsRef<[u8]>>(&self, contents: C) -> io::Result<()> {
        fs::write(self.file_path(), contents)
    }

    fn write_as_json<S: Serialize>(&self, serializable: S) -> Result<()> {
        let json = serde_json::to_string(&serializable).map_err(Error::CannotSerializeJSON)?;
        self.write(&json).map_err(Error::CannotWriteFile)?;
        Ok(())
    }

    fn write_as_json_pretty<S: Serialize>(&self, serializable: S) -> Result<()> {
        let json = serde_json::to_string_pretty(&serializable).map_err(Error::CannotSerializeJSON)?;
        self.write(&json).map_err(Error::CannotWriteFile)?;
        Ok(())
    }

    fn write_as_jsonlines<S: Serialize>(&self, serializable: &[S]) -> Result<()> {
        serde_jsonlines::write_json_lines(self.file_path(), serializable).map_err(Error::CannotSerializeJSONLines)?;
        Ok(())
    }
}

impl FileEx for Path {
    fn file_path(&self) -> &Path {
        self
    }
}
