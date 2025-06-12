use std::fmt::Display;

use skia_safe::{Color, Paint, PaintStyle, Path, Surface, surfaces};

use crate::maze::rect::{RectDirection, RectMaze, RectPosition};

use super::{Error, MazePaint};

pub trait CmdBoxCharset {
    fn horz_wall(&self) -> &str;
    fn horz_empty(&self) -> &str;
    fn vert_wall(&self) -> &str;
    fn vert_empty(&self) -> &str;
    fn select_corner(
        &self,
        has_west_wall: bool,
        has_north_wall: bool,
        has_east_wall: bool,
        has_south_wall: bool,
    ) -> &str;
}

#[derive(Debug, Clone, Copy)]
pub struct AsciiBoxCharset;

impl CmdBoxCharset for AsciiBoxCharset {
    fn horz_wall(&self) -> &str {
        "---"
    }

    fn horz_empty(&self) -> &str {
        "   "
    }

    fn vert_wall(&self) -> &str {
        "|"
    }

    fn vert_empty(&self) -> &str {
        " "
    }

    fn select_corner(
        &self,
        has_west_wall: bool,
        has_north_wall: bool,
        has_east_wall: bool,
        has_south_wall: bool,
    ) -> &str {
        match (has_west_wall, has_north_wall, has_east_wall, has_south_wall) {
            (false, false, false, false) => " ",
            _ => "+",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnicodeBoxCharset;

impl CmdBoxCharset for UnicodeBoxCharset {
    fn horz_wall(&self) -> &str {
        "\u{2501}"
    }

    fn horz_empty(&self) -> &str {
        " "
    }

    fn vert_wall(&self) -> &str {
        "\u{2503}"
    }

    fn vert_empty(&self) -> &str {
        " "
    }

    fn select_corner(
        &self,
        has_west_wall: bool,
        has_north_wall: bool,
        has_east_wall: bool,
        has_south_wall: bool,
    ) -> &str {
        let west = "\u{2578}";
        let north = "\u{2579}";
        let east = "\u{257a}";
        let south = "\u{257b}";
        let horz = "\u{2501}";
        let vert = "\u{2503}";
        let north_west = "\u{251b}";
        let north_east = "\u{2517}";
        let south_west = "\u{2513}";
        let south_east = "\u{250f}";
        let west_vert = "\u{252b}";
        let north_horz = "\u{253b}";
        let east_vert = "\u{2523}";
        let south_horz = "\u{2533}";
        let cross = "\u{254b}";

        match (has_west_wall, has_north_wall, has_east_wall, has_south_wall) {
            (false, false, false, false) => " ",
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

pub struct RectMazeCmdDisplay<'a, T: CmdBoxCharset>(pub &'a RectMaze, pub T);

impl<T: CmdBoxCharset> Display for RectMazeCmdDisplay<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(maze, charset) = self;
        let horz_wall = charset.horz_wall();
        let horz_empty = charset.horz_empty();
        let vert_wall = charset.vert_wall();
        let vert_empty = charset.vert_empty();

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
                let pos = RectPosition::new(r_ind, c_ind);
                let is_cell = maze.is_cell(&pos);
                let has_east_wall = if is_cell {
                    !maze.is_connected_to(&pos, RectDirection::North)
                } else {
                    pos.neighbor(RectDirection::North)
                        .is_some_and(|neighbor| maze.is_cell(&neighbor))
                };
                let has_north_wall = last_row_has_vert_wall[c_ind];
                let has_south_wall = if is_cell {
                    !maze.is_connected_to(&pos, RectDirection::West)
                } else {
                    pos.neighbor(RectDirection::West)
                        .is_some_and(|neighbor| maze.is_cell(&neighbor))
                };
                let corner = charset.select_corner(
                    has_west_wall,
                    has_north_wall,
                    has_east_wall,
                    has_south_wall,
                );
                ceil.push_str(corner);
                ceil.push_str(if has_east_wall { horz_wall } else { horz_empty });
                body.push_str(if has_south_wall {
                    vert_wall
                } else {
                    vert_empty
                });
                body.push_str(horz_empty);

                last_row_has_vert_wall[c_ind] = has_south_wall;
                has_west_wall = has_east_wall;
            }

            // Add the east border of the current row.
            let has_south_wall = width
                .checked_sub(1)
                .is_some_and(|c_ind| maze.is_cell(&RectPosition::new(r_ind, c_ind)));
            ceil.push_str(charset.select_corner(
                has_west_wall,
                east_column_has_north_wall,
                false,
                has_south_wall,
            ));
            body.push_str(if has_south_wall {
                vert_wall
            } else {
                vert_empty
            });
            east_column_has_north_wall = has_south_wall;
            writeln!(f, "{}", ceil)?;
            writeln!(f, "{}", body)?;
        }

        // Add the final south border.
        ceil.clear();
        let mut south_row_has_west_wall = false;
        for c_ind in 0..width {
            let has_east_wall = height
                .checked_sub(1)
                .is_some_and(|r_ind| maze.is_cell(&RectPosition::new(r_ind, c_ind)));
            ceil.push_str(charset.select_corner(
                south_row_has_west_wall,
                last_row_has_vert_wall[c_ind],
                has_east_wall,
                false,
            ));
            ceil.push_str(if has_east_wall { horz_wall } else { horz_empty });
            south_row_has_west_wall = has_east_wall;
        }
        ceil.push_str(charset.select_corner(
            south_row_has_west_wall,
            east_column_has_north_wall,
            false,
            false,
        ));
        write!(f, "{}", ceil)
    }
}

#[derive(Debug, Clone)]
pub struct RectMazePainter<'a> {
    maze: &'a RectMaze,
    wall_thickness: usize,
    cell_width: usize,
}

impl MazePaint for RectMazePainter<'_> {
    fn paint(&self) -> anyhow::Result<Surface, anyhow::Error> {
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
                let pos = RectPosition::new(r_ind, c_ind);
                let cell_x1 = cell_x0 + cell_interval;
                let has_north_wall = if self.maze.is_cell(&pos) {
                    !self.maze.is_connected_to(&pos, RectDirection::North)
                } else {
                    pos.neighbor(RectDirection::North)
                        .is_some_and(|neighbor| self.maze.is_cell(&neighbor))
                };
                let has_west_wall = if self.maze.is_cell(&pos) {
                    !self.maze.is_connected_to(&pos, RectDirection::West)
                } else {
                    pos.neighbor(RectDirection::West)
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
                .is_some_and(|c_ind| self.maze.is_cell(&RectPosition::new(r_ind, c_ind)))
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
                if self.maze.is_cell(&RectPosition::new(r_ind, c_ind)) {
                    path.move_to((cell_x0, cell_y0 + stroke_offset));
                    path.line_to((cell_x1 + wall_thickness, cell_y0 + stroke_offset));
                }

                cell_x0 = cell_x1;
            }
        }
        surface.canvas().draw_path(&path, &paint);

        Ok(surface)
    }
}

impl<'a> RectMazePainter<'a> {
    pub fn new(maze: &'a RectMaze, wall_thickness: usize, cell_width: usize) -> Self {
        Self {
            maze,
            wall_thickness,
            cell_width,
        }
    }
}
