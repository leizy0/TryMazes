use std::{fs::File, io::Write, path::Path};

use anyhow::Error as AnyError;
use clap::ValueEnum;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use skia_safe::{ColorSpace, EncodedImageFormat, ImageInfo, Surface, image::CachingHint};
use thiserror::Error;

pub mod circ;
pub mod hexa;
pub mod rect;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Failed to encode maze image to {0:?}")]
    ImageEncodeFailure(EncodedImageFormat),
    #[error("Failed to create surface")]
    CanNotCreateSurface,
    #[error("Failed to read pixels from maze image")]
    ReadPixelFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
pub enum SavePictureFormat {
    /// PNG file format
    PNG,
    /// JPEG file format
    JPEG,
}

pub trait MazePaint {
    fn paint(&self) -> Result<Surface, AnyError>;
}

pub struct MazePicture<'a, MP: MazePaint> {
    painter: &'a MP,
}

impl<'a, MP: MazePaint> MazePicture<'a, MP> {
    pub fn new(paint: &'a MP) -> Self {
        Self { painter: paint }
    }

    pub fn show(&self) -> Result<(), AnyError> {
        let mut surface = self.painter.paint()?;
        let image = surface.image_snapshot();
        let size = image.image_info().bounds().size();
        let mut pixels = vec![0u32; usize::try_from(size.width * size.height)?];
        let copy_info = ImageInfo::new_n32(size, image.alpha_type(), ColorSpace::new_srgb());
        let dst_row_bytes = usize::try_from(size.width)? * u32::BITS as usize / 8;
        if !image.read_pixels(
            &copy_info,
            pixels.as_mut_slice(),
            dst_row_bytes,
            (0, 0),
            CachingHint::Disallow,
        ) {
            return Err(Error::ReadPixelFailure.into());
        }

        Self::show_pixels(
            pixels.as_slice(),
            usize::try_from(size.width)?,
            usize::try_from(size.height)?,
        )?;
        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, format: SavePictureFormat) -> Result<(), AnyError> {
        let mut surface = self.painter.paint()?;

        let image = surface.image_snapshot();
        let mut context = surface.direct_context();
        let format = match format {
            SavePictureFormat::PNG => EncodedImageFormat::PNG,
            SavePictureFormat::JPEG => EncodedImageFormat::JPEG,
        };
        let data = image
            .encode(context.as_mut(), format, None)
            .ok_or(Error::ImageEncodeFailure(format))?;

        let mut file = File::create(path)?;
        file.write_all(data.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    fn show_pixels(pixels: &[u32], width: usize, height: usize) -> Result<(), AnyError> {
        let (wnd_width, wnd_height) = if width > height {
            (800, 600)
        } else {
            (600, 800)
        };

        let wnd_options = WindowOptions {
            resize: true,
            scale_mode: if width > wnd_width || height > wnd_height {
                ScaleMode::AspectRatioStretch
            } else {
                ScaleMode::Center
            },
            ..Default::default()
        };
        let mut window = Window::new(
            "Maze Demo - ESC to exit",
            wnd_width,
            wnd_height,
            wnd_options,
        )?;

        window.set_background_color(u8::MAX, u8::MAX, u8::MAX);
        // Limit to max ~60 fps update rate
        window.set_target_fps(60);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            window.update_with_buffer(pixels, width, height)?;
        }

        Ok(())
    }
}
