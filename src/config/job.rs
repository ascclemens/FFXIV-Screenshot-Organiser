use crate::{Result, config::Config, state::State};

use chrono::Local;

use anyhow::bail;

use image::{ImageFormat, ImageOutputFormat};

use std::{
  fs::OpenOptions,
  io::Write,
};

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

      let old_file_name = match f.file_name() {
        Some(f) => f,
        None => bail!("missing file name"),
      };

      let old_f = f.clone();
      *f = state.temp_dir.join(old_file_name);
      f.set_extension(to.extension());

      let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&f)?;

      match to {
        FileType::WebP { quality } => {
          let rgba = i.into_rgba();
          let flat = rgba.into_flat_samples();
          let stride = flat.layout.width_stride as u32 * flat.layout.width;
          let data = if quality < 0 {
            libwebp::WebPEncodeLosslessRGBA(&flat.samples, flat.layout.width, flat.layout.height, stride)?
          } else {
            let quality = std::cmp::min(100, quality);
            libwebp::WebPEncodeRGBA(&flat.samples, flat.layout.width, flat.layout.height, stride, f32::from(quality))?
          };
          dest.write_all(&data)?;
        },
        _ => i.write_to(&mut dest, to.as_image_output_format())?,
      }

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
      std::fs::copy(&f, &file_path)?;
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
  WebP {
    // less than 0 for lossless
    quality: i8,
  },
}

impl FileType {
  fn extension(self) -> &'static str {
    match self {
      FileType::Png => "png",
      FileType::Jpg { .. } => "jpg",
      FileType::Gif => "gif",
      FileType::Bmp => "bmp",
      FileType::Ico => "ico",
      FileType::WebP { .. } => "webp",
    }
  }

  fn as_image_format(self) -> ImageFormat {
    match self {
      FileType::Png => ImageFormat::Png,
      FileType::Jpg { .. } => ImageFormat::Jpeg,
      FileType::Gif => ImageFormat::Gif,
      FileType::Bmp => ImageFormat::Bmp,
      FileType::Ico => ImageFormat::Ico,
      FileType::WebP { .. } => ImageFormat::WebP,
    }
  }

  fn as_image_output_format(self) -> ImageOutputFormat {
    match self {
      FileType::Jpg { quality } => ImageOutputFormat::Jpeg(quality),
      _ => self.as_image_format().into(),
    }
  }
}
