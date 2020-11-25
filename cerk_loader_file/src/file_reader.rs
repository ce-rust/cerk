use anyhow::{Context, Result};
use std::env;
use std::fs;

pub(crate) fn read_file(filename: &str) -> Result<String> {
    fs::read_to_string(filename).with_context(|| {
        format!(
            "failed to read file {}, current dir is {}",
            filename,
            env::current_dir().unwrap_or_default().display()
        )
    })
}
