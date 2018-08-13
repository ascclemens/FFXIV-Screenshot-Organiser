use chrono::{DateTime, Utc};

use std::path::PathBuf;

#[derive(Debug)]
pub struct State {
  pub file_paths: Vec<PathBuf>,
  pub datetime: DateTime<Utc>,
}

impl State {
  pub fn new(file_path: PathBuf, datetime: DateTime<Utc>,) -> Self {
    State {
      datetime,
      file_paths: vec![file_path],
    }
  }
}
