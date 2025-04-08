use std::{cell::RefCell, fmt::Display, u8, u32};

use anyhow::{Error as AnyError, Result};
use clap::ValueEnum;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use skia_safe::{
    Color, ColorSpace, Data, EncodedImageFormat, ImageInfo, Paint, PaintStyle, Path, Surface,
    image::CachingHint, surfaces,
};
use thiserror::Error;

use crate::maze::{Direction, Maze};

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Failed to encode maze image to {0:?}")]
    ImageEncodeFailure(EncodedImageFormat),
    #[error("Failed to create surface")]
    CanNotCreateSurface,
    #[error("Failed to read pixels from maze image")]
    ReadPixelFailure,
}

#[derive(Debug, Clone, Copy)]
pub struct AsciiMazeDisplay<'a>(pub &'a Maze);

impl<'a> Display for AsciiMazeDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horz_wall = "---";
        let vert_wall = "|";
        let horz_empty = "   "; // 3 spaces.
        let vert_empty = " "; // 1 spaces.
        let corner = "+";

        let maze = self.0;
        let (width, height) = maze.size();
        let horz_line_len = width * (horz_wall.len() + corner.len()) + corner.len();
        let mut ceil_line = String::with_capacity(horz_line_len);
        let mut body_line = String::with_capacity(horz_line_len);
        for r_ind in 0..height {
            // The first cell in row.
            ceil_line.push_str(corner);
            body_line.push_str(vert_wall);
            for c_ind in 0..width {
                ceil_line.push_str(if maze.is_connect_to(r_ind, c_ind, Direction::North) {
                    horz_empty
                } else {
                    horz_wall
                });
                ceil_line.push_str(corner);

                body_line.push_str(horz_empty);
                body_line.push_str(if maze.is_connect_to(r_ind, c_ind, Direction::East) {
                    vert_empty
                } else {
                    vert_wall
                });
            }

            writeln!(f, "{}", ceil_line)?;
            writeln!(f, "{}", body_line)?;
            ceil_line.clear();
            body_line.clear();
        }

        // The bottom border.
        ceil_line.clear();
        for _ in 0..width {
            ceil_line.push_str(corner);
            ceil_line.push_str(horz_wall);
        }
        ceil_line.push_str(corner);
        write!(f, "{}", ceil_line)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
pub enum SavePictureFormat {
    /// PNG file format
    PNG,
    /// JPEG file format
    JPEG,
}

#[derive(Debug, Clone)]
pub struct GUIMazeShow<'a> {
    maze: &'a Maze,
    wall_thickness: usize,
    cell_width: usize,
    surface: RefCell<Option<Surface>>,
}

impl<'a> GUIMazeShow<'a> {
    pub fn new(maze: &'a Maze, wall_thickness: usize, cell_width: usize) -> Self {
        Self {
            maze,
            wall_thickness,
            cell_width,
            surface: RefCell::new(None),
        }
    }

    pub fn show(&self) -> Result<()> {
        self.init_surface()?;
        let mut surface_op = self.surface.borrow_mut();
        let surface = surface_op.as_mut().unwrap();
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

    pub fn image_data(&self, format: SavePictureFormat) -> Result<Data, AnyError> {
        self.init_surface()?;
        let mut surface_op = self.surface.borrow_mut();
        let surface = surface_op.as_mut().unwrap();

        let image = surface.image_snapshot();
        let mut context = surface.direct_context();
        let format = match format {
            SavePictureFormat::PNG => EncodedImageFormat::PNG,
            SavePictureFormat::JPEG => EncodedImageFormat::JPEG,
        };
        image
            .encode(context.as_mut(), format, None)
            .ok_or(Error::ImageEncodeFailure(format).into())
    }

    fn init_surface(&self) -> Result<()> {
        if self.surface.borrow().is_none() {
            let surface = self.paint()?;
            self.surface.borrow_mut().replace(surface);
        }

        Ok(())
    }

    fn paint(&self) -> Result<Surface> {
        let (width, height) = self.maze.size();
        let wall_thickness = i32::try_from(self.wall_thickness)?;
        let stroke_offset = wall_thickness / 2;
        let cell_interval = i32::try_from(self.cell_width + self.wall_thickness)?;
        let canvas_width = cell_interval * i32::try_from(width)? + wall_thickness;
        let canvas_height = cell_interval * i32::try_from(height)? + wall_thickness;
        let mut surface = surfaces::raster_n32_premul((canvas_width, canvas_height))
            .ok_or(Error::CanNotCreateSurface)?;
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        paint.set_anti_alias(false);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(u16::try_from(self.wall_thickness)?.into());
        surface.canvas().clear(Color::WHITE);

        let mut path = Path::new();
        let mut cell_y0 = 0;
        for r_ind in 0..height {
            let mut cell_x0 = 0;
            let cell_y1 = cell_y0 + cell_interval;
            for c_ind in 0..width {
                let cell_x1 = cell_x0 + cell_interval;
                if !self.maze.is_connect_to(r_ind, c_ind, Direction::North) {
                    path.move_to((cell_x0, cell_y0 + stroke_offset));
                    path.line_to((cell_x1 + wall_thickness, cell_y0 + stroke_offset));
                }

                if !self.maze.is_connect_to(r_ind, c_ind, Direction::West) {
                    path.move_to((cell_x0 + stroke_offset, cell_y0));
                    path.line_to((cell_x0 + stroke_offset, cell_y1 + wall_thickness));
                }

                cell_x0 += cell_interval;
            }
            // East border
            path.move_to((cell_x0 + stroke_offset, cell_y0));
            path.line_to((cell_x0 + stroke_offset, cell_y1 + wall_thickness));

            cell_y0 = cell_y1;
        }

        // South border
        let mut cell_x0 = 0;
        for _ in 0..width {
            let cell_x1 = cell_x0 + cell_interval;
            path.move_to((cell_x0, cell_y0 + stroke_offset));
            path.line_to((cell_x1 + wall_thickness, cell_y0 + stroke_offset));

            cell_x0 = cell_x1;
        }
        surface.canvas().draw_path(&path, &paint);

        Ok(surface)
    }

    fn show_pixels(pixels: &[u32], width: usize, height: usize) -> Result<()> {
        let (wnd_width, wnd_height) = if width > height {
            (800, 600)
        } else {
            (600, 800)
        };

        let mut wnd_options = WindowOptions::default();
        wnd_options.resize = true;
        wnd_options.scale_mode = if width > wnd_width || height > wnd_height {
            ScaleMode::AspectRatioStretch
        } else {
            ScaleMode::Center
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
            window.update_with_buffer(&pixels, width, height)?;
        }

        Ok(())
    }
}
