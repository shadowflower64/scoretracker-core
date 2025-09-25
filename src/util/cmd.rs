use crate::prompt_user;
use std::io::{self, Write};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
#[error("could not read string from stdin: {0}")]
pub struct AskError(#[from] io::Error);

pub fn ask<T, F: Fn(&str) -> Option<T>>(prompt: &str, validator: F) -> Result<T, AskError> {
    loop {
        prompt_user!("{prompt}: ");

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;

        if let Some(parsed) = validator(&buf) {
            return Ok(parsed);
        }
    }
}

pub fn ask_string(prompt: &str) -> Result<String, AskError> {
    ask(prompt, |answer| {
        let answer = answer.trim();
        if !answer.is_empty() { Some(answer.to_string()) } else { None }
    })
}

pub fn ask_yn(prompt: &str) -> Result<bool, AskError> {
    ask(&format!("{prompt} (y/n)"), |answer| {
        let answer = answer.trim().to_lowercase();
        if answer == "y" {
            Some(true)
        } else if answer == "n" {
            Some(false)
        } else {
            None
        }
    })
}

pub fn ask_parse<T: FromStr>(prompt: &str) -> Result<T, AskError> {
    ask(prompt, |answer| answer.trim().parse().ok())
}

pub fn ask_bool(prompt: &str) -> Result<bool, AskError> {
    ask_parse(prompt)
}

pub fn ask_i64(prompt: &str) -> Result<i64, AskError> {
    ask_parse(prompt)
}

pub fn ask_u64(prompt: &str) -> Result<u64, AskError> {
    ask_parse(prompt)
}

pub fn ask_f64(prompt: &str) -> Result<f64, AskError> {
    ask_parse(prompt)
}

pub fn ask_uuid(prompt: &str) -> Result<Uuid, AskError> {
    ask_parse(prompt)
}
