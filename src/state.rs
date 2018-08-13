use chrono::{DateTime, Utc};

use std::path::PathBuf;

#[derive(Debug)]
pub struct State {
  pub file_paths: Vec<PathBuf>,
  pub datetime: DateTime<Utc>,
  pub temp_dir: PathBuf,
}

impl State {
  pub fn new(file_path: PathBuf, datetime: DateTime<Utc>, temp_dir: PathBuf) -> Self {
    State {
      datetime,
      temp_dir,
      file_paths: vec![file_path],
    }
  }
}
