use skia_safe::{Color, Paint, Path, Surface, surfaces};

use crate::maze::tri::{TriDirection, TriMaze, TriPosition};

use super::{Error, MazePaint};

pub struct TriMazePainter<'a> {
    maze: &'a TriMaze,
    tri_cell_height: u16,
    wall_thickness: u16,
}

impl MazePaint for TriMazePainter<'_> {
    fn paint(&self) -> Result<Surface, anyhow::Error> {
        let maze = self.maze;
        let (maze_width, maze_height) = maze.size();
        let wall_thickness = f32::from(self.wall_thickness);
        let tri_cell_height = f32::from(self.tri_cell_height);
        let sqrt_3 = 3f32.sqrt();
        let cell_vert_interval = tri_cell_height + wall_thickness * 1.5;
        let cell_horz_interval = cell_vert_interval / sqrt_3;
        let canvas_vert_offset = wall_thickness;
        let canvas_horz_offset = wall_thickness / sqrt_3;
        let pic_width = (f32::from(u16::try_from(maze_width + 1)?) * cell_horz_interval
            + canvas_horz_offset * 2.0)
            .ceil() as i32;
        let pic_height = (f32::from(u16::try_from(maze_height)?) * cell_vert_interval
            + canvas_vert_offset * 2.0)
            .ceil() as i32;
        let mut surface = surfaces::raster_n32_premul((pic_width, pic_height))
            .ok_or(Error::CanNotCreateSurface)?;
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::PaintStyle::Stroke);
        paint.set_stroke_width(wall_thickness);

        surface.canvas().save();
        surface
            .canvas()
            .translate((canvas_horz_offset, canvas_vert_offset));
        surface.canvas().clear(Color::WHITE);
        let mut path = Path::new();
        let mut top_center_y = 0f32;
        let has_wall_at = |is_cell, pos: &TriPosition, dir| -> bool {
            (is_cell && !maze.is_connected_to(pos, dir))
                || (!is_cell
                    && pos
                        .neighbor(dir)
                        .is_some_and(|neighbor| maze.is_cell(&neighbor)))
        };
        for r in 0..maze_height {
            let mut top_center_x = cell_horz_interval;
            for c in 0..maze_width {
                let pos = TriPosition::new(r, c);
                let is_cell = maze.is_cell(&pos);
                if maze.is_angle_up(&pos) {
                    let bot_left_x = top_center_x - cell_horz_interval;
                    let bot_left_y = top_center_y + cell_vert_interval;
                    let bot_right_x = top_center_x + cell_horz_interval;
                    let bot_right_y = bot_left_y;
                    if has_wall_at(is_cell, &pos, TriDirection::Northwest) {
                        path.move_to((top_center_x, top_center_y));
                        path.line_to((bot_left_x, bot_left_y));
                    }

                    if has_wall_at(is_cell, &pos, TriDirection::South) {
                        path.move_to((bot_left_x, bot_left_y));
                        path.line_to((bot_right_x, bot_right_y));
                    }

                    if c == maze_width - 1 && is_cell {
                        path.move_to((top_center_x, top_center_y));
                        path.line_to((bot_right_x, bot_right_y));
                    }
                } else {
                    let top_left_x = top_center_x - cell_horz_interval;
                    let top_left_y = top_center_y;
                    let top_right_x = top_center_x + cell_horz_interval;
                    let top_right_y = top_left_y;
                    let bot_center_x = top_center_x;
                    let bot_center_y = top_center_y + cell_vert_interval;
                    if has_wall_at(is_cell, &pos, TriDirection::SouthWest) {
                        path.move_to((top_left_x, top_left_y));
                        path.line_to((bot_center_x, bot_center_y));
                    }

                    if r == 0 && is_cell {
                        path.move_to((top_left_x, top_left_y));
                        path.line_to((top_right_x, top_right_y));
                    }

                    if c == maze_width - 1 && is_cell {
                        path.move_to((top_right_x, top_right_y));
                        path.line_to((bot_center_x, bot_center_y));
                    }
                }

                top_center_x += cell_horz_interval;
            }

            top_center_y += cell_vert_interval;
        }
        surface.canvas().draw_path(&path, &paint);
        surface.canvas().restore();

        Ok(surface)
    }
}

impl<'a> TriMazePainter<'a> {
    pub fn new(maze: &'a TriMaze, tri_cell_height: u16, wall_thickness: u16) -> Self {
        Self {
            maze,
            tri_cell_height,
            wall_thickness,
        }
    }
}
