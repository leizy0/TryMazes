use skia_safe::{Color, Paint, Path, Surface, surfaces};

use crate::maze::hexa::{HexaDirection, HexaMaze, HexaPosition};

use super::{Error, MazePaint};

#[derive(Debug)]
pub struct HexaMazePainter<'a> {
    maze: &'a HexaMaze,
    hexa_cell_height: u16,
    wall_thickness: u16,
}

impl MazePaint for HexaMazePainter<'_> {
    fn paint(&self) -> Result<Surface, anyhow::Error> {
        let maze = self.maze;
        let (maze_width, maze_height) = maze.size();
        let cell_height = f32::from(self.hexa_cell_height);
        let wall_thickness = f32::from(self.wall_thickness);
        let cell_vert_interval = cell_height + wall_thickness;
        let sqrt_3 = 3f32.sqrt();
        let cell_radius = cell_vert_interval / sqrt_3;
        let pic_width = (wall_thickness / sqrt_3 * 2.0
            + (3.0 * f32::from(u16::try_from(maze_width)?) + 1.0) / 2.0 * cell_radius)
            .ceil() as i32;
        let pic_height = (wall_thickness
            + (f32::from(u16::try_from(maze_height)?) + 0.5) * cell_vert_interval)
            .ceil() as i32;
        let mut surface = surfaces::raster_n32_premul((pic_width, pic_height))
            .ok_or(Error::CanNotCreateSurface)?;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::BLACK);
        paint.set_stroke_width(wall_thickness);
        paint.set_style(skia_safe::PaintStyle::Stroke);

        let stroke_horz_offset = wall_thickness / sqrt_3;
        let stroke_vert_offset = wall_thickness / 2.0;
        surface.canvas().clear(Color::WHITE);
        surface.canvas().save();
        surface
            .canvas()
            .translate((stroke_horz_offset, stroke_vert_offset));
        let mut center_y = cell_vert_interval / 2.0;
        // Paint vertices(northeast, northwest, west, southwest)
        let paint_hex_vertex_degrees = [-60f32, -120f32, -180f32, -240f32];
        let mut paint_hex_vertices_x = [0f32; 4];
        let mut paint_hex_vertices_y = [0f32; 4];
        let paint_hex_edge_dirs = [
            HexaDirection::North,
            HexaDirection::NorthWest,
            HexaDirection::SouthWest,
        ];
        let mut path = Path::new();
        for r in 0..maze_height {
            let mut center_x = cell_radius;
            for c in 0..maze_width {
                let pos = HexaPosition::new(r, c);
                for (i, deg) in paint_hex_vertex_degrees.iter().copied().enumerate() {
                    let rad = deg.to_radians();
                    paint_hex_vertices_x[i] = center_x + cell_radius * rad.cos();
                    paint_hex_vertices_y[i] = center_y + cell_radius * rad.sin();
                }

                if maze.is_cell(&pos) {
                    if r == 0 && c % 2 == 0 {
                        // Add the northeast edge of cells in the first row.
                        let east_vertex_x = center_x + cell_radius;
                        let east_vertex_y = center_y;
                        path.move_to((east_vertex_x, east_vertex_y));
                        path.line_to((paint_hex_vertices_x[0], paint_hex_vertices_y[0]));
                    }

                    for (ind, dir) in paint_hex_edge_dirs.iter().copied().enumerate() {
                        if !maze.is_connected_to(&pos, dir) {
                            path.move_to((paint_hex_vertices_x[ind], paint_hex_vertices_y[ind]));
                            path.line_to((
                                paint_hex_vertices_x[ind + 1],
                                paint_hex_vertices_y[ind + 1],
                            ));
                        }
                    }

                    // Add the northeast edge and the southeast edge of the last cells in every row.
                    if c == maze_width - 1 {
                        let east_vertex_x = center_x + cell_radius;
                        let east_vertex_y = center_y;
                        let southeast_vertex_x = center_x + cell_radius / 2.0;
                        let southeast_vertex_y = center_y + cell_vert_interval / 2.0;
                        path.move_to((southeast_vertex_x, southeast_vertex_y));
                        path.line_to((east_vertex_x, east_vertex_y));
                        path.line_to((paint_hex_vertices_x[0], paint_hex_vertices_y[0]));
                    }

                    // Add the south edge and the southeast edge of the last cells in every column.
                    if r == maze_height - 1 {
                        let southwest_vertex_x = center_x - cell_radius / 2.0;
                        let southwest_vertex_y = center_y + cell_vert_interval / 2.0;
                        let southeast_vertex_x = center_x + cell_radius / 2.0;
                        let southeast_vertex_y = center_y + cell_vert_interval / 2.0;
                        if c % 2 == 0 {
                            path.move_to((southwest_vertex_x, southwest_vertex_y));
                            path.line_to((southeast_vertex_x, southeast_vertex_y));
                        } else {
                            let east_vertex_x = center_x + cell_radius;
                            let east_vertex_y = center_y;
                            path.move_to((southwest_vertex_x, southwest_vertex_y));
                            path.line_to((southeast_vertex_x, southeast_vertex_y));
                            path.line_to((east_vertex_x, east_vertex_y));
                        }
                    }
                } else {
                    // Add the walls between a cell position and a non-cell position.
                    for (ind, dir) in paint_hex_edge_dirs.iter().copied().enumerate() {
                        if pos
                            .neighbor(dir)
                            .is_some_and(|neighbor| maze.is_cell(&neighbor))
                        {
                            path.move_to((paint_hex_vertices_x[ind], paint_hex_vertices_y[ind]));
                            path.line_to((
                                paint_hex_vertices_x[ind + 1],
                                paint_hex_vertices_y[ind + 1],
                            ));
                        }
                    }
                }

                center_x += cell_radius * 1.5;
                // The y coordinates of the center plot a zigzag line.
                center_y += if c % 2 == 0 {
                    cell_vert_interval / 2.0
                } else {
                    -cell_vert_interval / 2.0
                };
            }

            // Adjust the center y coordinate of the first cell in the next row, due to the last cell's position(the oddity of the row width).
            center_y += if maze_width % 2 == 0 {
                cell_vert_interval
            } else {
                cell_vert_interval / 2.0
            };
        }
        surface.canvas().draw_path(&path, &paint);
        surface.canvas().restore();

        Ok(surface)
    }
}

impl<'a> HexaMazePainter<'a> {
    pub fn new(maze: &'a HexaMaze, hexa_cell_height: u16, wall_thickness: u16) -> Self {
        Self {
            maze,
            hexa_cell_height,
            wall_thickness,
        }
    }
}
