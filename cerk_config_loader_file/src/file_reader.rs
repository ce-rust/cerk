use anyhow::{Context, Result};
use std::fs;

pub(crate) fn read_file(filename: &str) -> Result<String> {
    fs::read_to_string(filename).with_context(|| format!("failed to read file {}", filename))
}
