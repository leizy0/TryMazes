use std::{cell::RefCell, fmt::Display};

use anyhow::{Error as AnyError, Result};
use clap::ValueEnum;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use skia_safe::{
    Color, ColorSpace, Data, EncodedImageFormat, ImageInfo, Paint, PaintStyle, Path, Surface,
    image::CachingHint, surfaces,
};
use thiserror::Error;

use crate::maze::{Direction, Maze, Position};

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

impl Display for AsciiMazeDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horz_wall = "---";
        let vert_wall = "|";
        let horz_empty = "   "; // 3 spaces.
        let vert_empty = " "; // 1 space.
        let corner = "+";

        let maze = self.0;
        let (width, height) = maze.size();
        let mut ceil_line = String::new();
        let mut body_line = String::new();
        for r_ind in 0..height {
            ceil_line.clear();
            body_line.clear();
            for c_ind in 0..width {
                let pos = Position::new(r_ind, c_ind);
                ceil_line.push_str(corner);
                if maze.is_cell(&pos) {
                    ceil_line.push_str(if !maze.is_connect_to(&pos, Direction::North) {
                        horz_wall
                    } else {
                        horz_empty
                    });
                    body_line.push_str(if !maze.is_connect_to(&pos, Direction::West) {
                        vert_wall
                    } else {
                        vert_empty
                    });
                } else {
                    ceil_line.push_str(
                        if pos
                            .neighbor(Direction::North)
                            .is_some_and(|neighbor| maze.is_cell(&neighbor))
                        {
                            horz_wall
                        } else {
                            horz_empty
                        },
                    );
                    body_line.push_str(
                        if pos
                            .neighbor(Direction::West)
                            .is_some_and(|neighbor| maze.is_cell(&neighbor))
                        {
                            vert_wall
                        } else {
                            vert_empty
                        },
                    );
                }
                body_line.push_str(horz_empty);
            }
            ceil_line.push_str(corner);
            body_line.push_str(if maze.is_cell(&Position::new(r_ind, width - 1)) {
                vert_wall
            } else {
                vert_empty
            });

            writeln!(f, "{}", ceil_line)?;
            writeln!(f, "{}", body_line)?;
        }

        // The bottom border.
        ceil_line.clear();
        for c_ind in 0..width {
            ceil_line.push_str(corner);
            ceil_line.push_str(
                if Position::new(height, c_ind)
                    .neighbor(Direction::North)
                    .is_some_and(|neighbor| maze.is_cell(&neighbor))
                {
                    horz_wall
                } else {
                    horz_empty
                },
            );
        }
        ceil_line.push_str(corner);
        write!(f, "{}", ceil_line)
    }
}

#[derive(Debug, Clone)]
pub struct UnicodeDisplay<'a>(pub &'a Maze);

impl Display for UnicodeDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horz = '\u{2501}';
        let vert = '\u{2503}';

        let maze = self.0;
        let (width, height) = maze.size();
        let mut ceil = String::new();
        let mut body = String::new();
        let mut last_row_has_vert_wall = vec![false; width];
        let mut east_column_has_north_wall = false;
        for r_ind in 0..height {
            ceil.clear();
            body.clear();
            let mut has_west_wall = false;
            for c_ind in 0..width {
                let pos = Position::new(r_ind, c_ind);
                let is_cell = maze.is_cell(&pos);
                let has_east_wall = if is_cell {
                    !maze.is_connect_to(&pos, Direction::North)
                } else {
                    pos.neighbor(Direction::North)
                        .is_some_and(|neighbor| maze.is_cell(&neighbor))
                };
                let has_north_wall = last_row_has_vert_wall[c_ind];
                let has_south_wall = if is_cell {
                    !maze.is_connect_to(&pos, Direction::West)
                } else {
                    pos.neighbor(Direction::West)
                        .is_some_and(|neighbor| maze.is_cell(&neighbor))
                };
                let corner = Self::select_corner(
                    has_west_wall,
                    has_north_wall,
                    has_east_wall,
                    has_south_wall,
                );
                ceil.push(corner);
                ceil.push(if has_east_wall { horz } else { ' ' });
                body.push(if has_south_wall { vert } else { ' ' });
                body.push(' ');

                last_row_has_vert_wall[c_ind] = has_south_wall;
                has_west_wall = has_east_wall;
            }

            let has_south_wall = width
                .checked_sub(1)
                .is_some_and(|c_ind| maze.is_cell(&Position::new(r_ind, c_ind)));
            ceil.push(Self::select_corner(
                has_west_wall,
                east_column_has_north_wall,
                false,
                has_south_wall,
            ));
            body.push(if has_south_wall { vert } else { ' ' });
            east_column_has_north_wall = has_south_wall;
            writeln!(f, "{}", ceil)?;
            writeln!(f, "{}", body)?;
        }

        ceil.clear();
        let mut south_row_has_west_wall = false;
        for c_ind in 0..width {
            let has_east_wall = height
                .checked_sub(1)
                .is_some_and(|r_ind| maze.is_cell(&Position::new(r_ind, c_ind)));
            ceil.push(Self::select_corner(
                south_row_has_west_wall,
                last_row_has_vert_wall[c_ind],
                has_east_wall,
                false,
            ));
            ceil.push(if has_east_wall { horz } else { ' ' });
            south_row_has_west_wall = has_east_wall;
        }
        ceil.push(Self::select_corner(
            south_row_has_west_wall,
            east_column_has_north_wall,
            false,
            false,
        ));
        write!(f, "{}", ceil)
    }
}

impl UnicodeDisplay<'_> {
    fn select_corner(
        has_west_wall: bool,
        has_north_wall: bool,
        has_east_wall: bool,
        has_south_wall: bool,
    ) -> char {
        let west = '\u{2578}';
        let north = '\u{2579}';
        let east = '\u{257a}';
        let south = '\u{257b}';
        let horz = '\u{2501}';
        let vert = '\u{2503}';
        let north_west = '\u{251b}';
        let north_east = '\u{2517}';
        let south_west = '\u{2513}';
        let south_east = '\u{250f}';
        let west_vert = '\u{252b}';
        let north_horz = '\u{253b}';
        let east_vert = '\u{2523}';
        let south_horz = '\u{2533}';
        let cross = '\u{254b}';

        match (has_west_wall, has_north_wall, has_east_wall, has_south_wall) {
            (false, false, false, false) => ' ',
            (true, false, false, false) => west,
            (false, true, false, false) => north,
            (false, false, true, false) => east,
            (false, false, false, true) => south,
            (true, false, true, false) => horz,
            (false, true, false, true) => vert,
            (true, true, false, false) => north_west,
            (false, true, true, false) => north_east,
            (true, false, false, true) => south_west,
            (false, false, true, true) => south_east,
            (true, true, false, true) => west_vert,
            (true, true, true, false) => north_horz,
            (false, true, true, true) => east_vert,
            (true, false, true, true) => south_horz,
            (true, true, true, true) => cross,
        }
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
                let pos = Position::new(r_ind, c_ind);
                let cell_x1 = cell_x0 + cell_interval;
                let has_north_wall = if self.maze.is_cell(&pos) {
                    !self.maze.is_connect_to(&pos, Direction::North)
                } else {
                    pos.neighbor(Direction::North)
                        .is_some_and(|neighbor| self.maze.is_cell(&neighbor))
                };
                let has_west_wall = if self.maze.is_cell(&pos) {
                    !self.maze.is_connect_to(&pos, Direction::West)
                } else {
                    pos.neighbor(Direction::West)
                        .is_some_and(|neighbor| self.maze.is_cell(&neighbor))
                };
                if has_north_wall {
                    path.move_to((cell_x0, cell_y0 + stroke_offset));
                    path.line_to((cell_x1 + wall_thickness, cell_y0 + stroke_offset));
                }

                if has_west_wall {
                    path.move_to((cell_x0 + stroke_offset, cell_y0));
                    path.line_to((cell_x0 + stroke_offset, cell_y1 + wall_thickness));
                }

                cell_x0 += cell_interval;
            }
            // East border
            if width
                .checked_sub(1)
                .is_some_and(|c_ind| self.maze.is_cell(&Position::new(r_ind, c_ind)))
            {
                path.move_to((cell_x0 + stroke_offset, cell_y0));
                path.line_to((cell_x0 + stroke_offset, cell_y1 + wall_thickness));
            }

            cell_y0 = cell_y1;
        }

        // South border
        if let Some(r_ind) = height.checked_sub(1) {
            let mut cell_x0 = 0;
            for c_ind in 0..width {
                let cell_x1 = cell_x0 + cell_interval;
                if self.maze.is_cell(&Position::new(r_ind, c_ind)) {
                    path.move_to((cell_x0, cell_y0 + stroke_offset));
                    path.line_to((cell_x1 + wall_thickness, cell_y0 + stroke_offset));
                }

                cell_x0 = cell_x1;
            }
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
