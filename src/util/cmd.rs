use std::io::{self, Write};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("could not read string from stdin: {0}")]
pub struct AskError(#[from] io::Error);

pub fn ask(prompt: &str) -> Result<String, AskError> {
    loop {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let answer = buf.trim();
        if !answer.is_empty() {
            return Ok(answer.to_string());
        }
    }
}

pub fn ask_yn(prompt: &str) -> Result<bool, AskError> {
    loop {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let answer = buf.trim().to_lowercase();
        if answer == "y" {
            return Ok(true);
        } else if answer == "n" {
            return Ok(false);
        }
    }
}
