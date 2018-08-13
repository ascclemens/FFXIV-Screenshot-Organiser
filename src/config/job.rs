use crate::{Result, config::Config, state::State};

use chrono::Local;

use failure::bail;

use image::{ImageFormat, ImageOutputFormat};

use std::fs::OpenOptions;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "job", content = "options")]
pub enum Job {
  Convert {
    to: FileType,
    #[serde(default)]
    keep_original: bool,
  },
  Move {
    to: String,
    #[serde(default)]
    local: Option<bool>,
  },
}

impl Job {
  pub fn execute(&self, config: &Config, state: &mut State) -> Result<()> {
    match *self {
      Job::Convert { to, keep_original } => Job::convert(state, to, keep_original),
      Job::Move { ref to, local } => Job::move_(config, state, to, local),
    }
  }

  fn convert(state: &mut State, to: FileType, keep: bool) -> Result<()> {
    let mut add = None;
    for f in &mut state.file_paths {
      let i = image::open(&f)?;

      let old_f = f.clone();
      f.set_extension(to.extension());

      let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&f)?;

      i.write_to(&mut dest, to.as_image_output_format())?;

      if keep {
        add = Some(old_f);
      } else {
        std::fs::remove_file(old_f)?;
      }
    }

    if let Some(f) = add {
      state.file_paths.push(f);
    }

    Ok(())
  }

  fn move_(config: &Config, state: &mut State, to: &str, local: Option<bool>) -> Result<()> {
    for f in &mut state.file_paths {
      let ext = match f.extension() {
        Some(e) => e,
        None => bail!("missing extension on {}", f.to_string_lossy()),
      };
      let dt = if local.unwrap_or(true) {
        state.datetime.with_timezone(&Local).format(to)
      } else {
        state.datetime.format(to)
      };
      let path = format!("{}.{}", dt, ext.to_string_lossy());
      let file_path = config.options.screenshots_dir.join(path);
      if let Some(p) = file_path.parent() {
        std::fs::create_dir_all(p)?;
      }
      std::fs::rename(&f, &file_path)?;
      *f = file_path;
    }

    Ok(())
  }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "format")]
pub enum FileType {
  Png,
  Jpg {
    quality: u8,
  },
  Gif,
  Bmp,
  Ico,
}

impl FileType {
  fn extension(self) -> &'static str {
    match self {
      FileType::Png => "png",
      FileType::Jpg { .. } => "jpg",
      FileType::Gif => "gif",
      FileType::Bmp => "bmp",
      FileType::Ico => "ico",
    }
  }

  fn as_image_format(self) -> ImageFormat {
    match self {
      FileType::Png => ImageFormat::PNG,
      FileType::Jpg { .. } => ImageFormat::JPEG,
      FileType::Gif => ImageFormat::GIF,
      FileType::Bmp => ImageFormat::BMP,
      FileType::Ico => ImageFormat::ICO,
    }
  }

  fn as_image_output_format(self) -> ImageOutputFormat {
    match self {
      FileType::Jpg { quality } => ImageOutputFormat::JPEG(quality),
      _ => self.as_image_format().into(),
    }
  }
}
