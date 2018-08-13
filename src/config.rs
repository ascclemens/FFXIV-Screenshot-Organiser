mod job;

pub use self::job::Job;

use regex::Regex;

use serde::{Deserialize, Deserializer};

use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub options: Options,
  pub pipeline: Vec<Job>,
}

#[derive(Debug, Deserialize)]
pub struct Options {
  pub screenshots_dir: PathBuf,
  #[serde(rename = "match")]
  #[serde(deserialize_with = "regex_vec")]
  pub patterns: Vec<Regex>,
  pub event_delay: i64,
}

fn regex_vec<'de, D>(deserialiser: D) -> Result<Vec<Regex>, D::Error>
  where D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  struct Wrapper(#[serde(with = "serde_regex")] Regex);

  let v = Vec::deserialize(deserialiser)?;
  Ok(v.into_iter().map(|Wrapper(r)| r).collect())
}
