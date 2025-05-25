use anyhow::Error as AnyError;
use skia_safe::{Color, Paint, PaintStyle, Path, PathDirection, Rect, Surface, surfaces};

use crate::maze::circ::{CircMaze, CircPosition};

use super::{Error, MazePaint};

pub struct CircMazePainter<'a> {
    maze: &'a CircMaze,
    ring_interval_width: usize,
    wall_thickness: usize,
}

impl MazePaint for CircMazePainter<'_> {
    fn paint(&self) -> Result<Surface, AnyError> {
        let maze = self.maze;
        let rings_n = maze.rings_n();
        let ring_interval = i32::try_from(self.ring_interval_width + self.wall_thickness)?;
        let total_radius = ring_interval * i32::try_from(rings_n)?;
        let mut surface = surfaces::raster_n32_premul((total_radius * 2, total_radius * 2))
            .ok_or(Error::CanNotCreateSurface)?;
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        paint.set_anti_alias(true);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(u16::try_from(self.wall_thickness)?.into());

        let mut path = Path::new();
        surface.canvas().clear(Color::WHITE);
        surface.canvas().save();
        surface.canvas().translate((total_radius, total_radius));
        let mut cur_radius = f32::from(u16::try_from(self.ring_interval_width)?)
            + f32::from(u16::try_from(self.wall_thickness)?) / 2.0;
        let ring_interval = f32::from(u16::try_from(ring_interval)?);
        for ring in 1..rings_n {
            let ring_cells_n = maze.ring_cells_n(ring);
            let cell_angle_interval = 360.0 / f32::from(u16::try_from(ring_cells_n)?);
            let mut cur_angle = 0f32;
            for cell in 0..ring_cells_n {
                let pos = CircPosition::new(ring, cell);
                if !maze.is_connected_inward(&pos) {
                    path.add_arc(
                        Rect::from_ltrb(-cur_radius, -cur_radius, cur_radius, cur_radius),
                        cur_angle,
                        cell_angle_interval,
                    );
                }

                // Rotate to angle of the clockwise wall(also the start angle of the next cell).
                cur_angle += cell_angle_interval;
                if !maze.is_connected_clockwise(&pos) {
                    let clockwise_wall_inward_x = cur_radius * cur_angle.to_radians().cos();
                    let clockwise_wall_inward_y = cur_radius * cur_angle.to_radians().sin();
                    path.move_to((clockwise_wall_inward_x, clockwise_wall_inward_y));
                    let clockwise_wall_outward_x =
                        (cur_radius + ring_interval) * cur_angle.to_radians().cos();
                    let clockwise_wall_outward_y =
                        (cur_radius + ring_interval) * cur_angle.to_radians().sin();
                    path.line_to((clockwise_wall_outward_x, clockwise_wall_outward_y));
                }
            }

            cur_radius += ring_interval;
        }
        path.add_circle((0f32, 0f32), cur_radius, PathDirection::CW);
        surface.canvas().draw_path(&path, &paint);
        surface.canvas().restore();

        Ok(surface)
    }
}

impl<'a> CircMazePainter<'a> {
    pub fn new(maze: &'a CircMaze, ring_interval_width: usize, wall_thickness: usize) -> Self {
        Self {
            maze,
            ring_interval_width,
            wall_thickness,
        }
    }
}
