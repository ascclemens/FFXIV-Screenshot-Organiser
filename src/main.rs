#![feature(rust_2018_preview, iterator_find_map, use_extern_macros)]

#[macro_use] extern crate crossbeam_channel;
#[macro_use] extern crate serde_derive;

mod config;
mod state;

use crate::{
  config::Config,
  state::State,
};

use crossbeam_channel::Receiver;

use chrono::{DateTime, Duration, Local, TimeZone, Utc};

use failure::Error;

use notify::{DebouncedEvent, RecursiveMode, Watcher};

use rayon::prelude::*;

use tempdir::TempDir;

use std::{
  fs::{self, DirEntry},
  path::PathBuf,
  sync::{
    Arc,
    mpsc::{self, TryRecvError},
  },
};

pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
  println!("Starting FFXIV Screenshot Organiser.");

  let ctrlc_rx = set_ctrlc_handler(num_cpus::get() + 1)?;

  let config_path = match std::env::args().nth(1) {
    Some(x) => x,
    None => "config.json".into(),
  };
  println!("Attempting to read config from `{}`.", config_path);

  let f = fs::File::open(config_path)?;
  let config: Config = serde_json::from_reader(f)?;

  println!("Config successfully read.");

  let screenshots_dir = config.options.screenshots_dir.canonicalize()?;

  println!("Screenshots are located at `{}`.", screenshots_dir.to_string_lossy());

  let mut handles = Vec::with_capacity(num_cpus::get() + 1);

  let (tx, rx) = mpsc::channel();
  let (event_tx, event_rx) = crossbeam_channel::unbounded();

  let temp_dir = TempDir::new("fso-")?;
  let temp_path = temp_dir.path().to_owned();
  println!("Storing temporary files in `{}`.", temp_path.to_string_lossy());

  let mut watcher = notify::watcher(
    tx,
    Duration::milliseconds(config.options.event_delay).to_std().unwrap(),
  )?;

  watcher.watch(
    screenshots_dir.to_string_lossy().to_string(),
    RecursiveMode::NonRecursive
  )?;

  {
    let thread_ctrlc_rx = ctrlc_rx.clone();
    handles.push(std::thread::spawn(move || {
      let tick = crossbeam_channel::tick(Duration::milliseconds(50).to_std().unwrap());
      loop {
        select! {
          recv(tick, _) => match rx.try_recv() {
              Ok(e) => event_tx.send(e),
              Err(TryRecvError::Empty) => {},
              Err(TryRecvError::Disconnected) => break,
          },
          recv(thread_ctrlc_rx, _) => break,
        }
      }
    }));
  }

  println!("Waiting for new files on {} thread(s).", num_cpus::get());

  let config = Arc::new(config);
  let mut handles = Vec::with_capacity(num_cpus::get());
  for i in 0..num_cpus::get() {
    let thread_ctrlc_rx = ctrlc_rx.clone();
    let thread_event_rx = event_rx.clone();
    let temp_path = temp_path.clone();
    let thread_config = Arc::clone(&config);

    let handle = std::thread::spawn(move || {
      loop {
        select! {
          recv(thread_ctrlc_rx, _) => break,
          recv(thread_event_rx, e) => {
            match e {
              Some(DebouncedEvent::Create(p)) => if let Err(e) = handle(&thread_config, Some(i), temp_path.clone(), p) {
                eprintln!("{}", e);
              },
              Some(_) => {},
              None => eprintln!("{:#?}", e),
            }
          },
        }
      }
      println!("Thread {} shutting down.", i);
    });
    handles.push(handle);
  }

  println!("Collecting existing files.");

  let mut existing_files: Vec<DirEntry> = std::fs::read_dir(&screenshots_dir)?.collect::<std::result::Result<_, _>>()?;
  existing_files.retain(|x| x.path().is_file());

  println!("Processing {} existing file(s).", existing_files.len());

  existing_files.into_par_iter().for_each(|entry| {
    if let Err(e) = handle(&config, None, temp_path.clone(), entry.path()) {
      eprintln!("{}", e);
    }
  });

  println!("Done!");

  for handle in handles {
    handle.join().ok();
  }

  println!("Exiting.");

  Ok(())
}

fn handle(config: &Config, thread: Option<usize>, temp_dir: PathBuf, p: PathBuf) -> Result<()> {
  let screenshots_dir = config.options.screenshots_dir.canonicalize()?;

  // if the path doesn't exist, ignore
  // no need to print out an error – usually a result of duplicate events
  if !p.exists() {
    return Ok(());
  }

  // if the path isn't at the top level, ignore
  if p.strip_prefix(&screenshots_dir).unwrap().components().count() != 1 {
    return Ok(());
  }

  // ignore paths without file names
  let file_name: String = match p.file_name() {
    Some(f) => f.to_string_lossy().to_string(),
    None => return Ok(()),
  };

  // ignore paths that don't match
  let time = match parse_screenshot_name(&config, &file_name) {
    Some(t) => t,
    None => return Ok(()),
  };

  match thread {
    Some(i) => println!("Handling `{}` on thread {}.", file_name, i),
    None => println!("Handling `{}`.", file_name),
  }

  // execute the jobs in the pipeline
  let mut state = State::new(p, time, temp_dir);
  for job in &config.pipeline {
    job.execute(&config, &mut state)?;
  }

  Ok(())
}

fn parse_screenshot_name(config: &Config, s: &str) -> Option<DateTime<Utc>> {
  let caps = config.options.patterns.iter().find_map(|p| p.captures(s))?;
  let dt = Local
    .ymd(
      caps.name("year")?.as_str().parse().ok()?,
      caps.name("month")?.as_str().parse().ok()?,
      caps.name("day")?.as_str().parse().ok()?,
    )
    .and_hms(
      caps.name("hour")?.as_str().parse().ok()?,
      caps.name("minute")?.as_str().parse().ok()?,
      caps.name("second")?.as_str().parse().ok()?,
    );
  Some(dt.with_timezone(&Utc))
}

fn set_ctrlc_handler(num_threads: usize) -> Result<Receiver<()>> {
  let (tx, rx) = crossbeam_channel::unbounded();

  ctrlc::set_handler(move || {
    println!("Received interrupt.");
    for _ in 0..num_threads {
      tx.send(());
    }
  })?;

  Ok(rx)
}
