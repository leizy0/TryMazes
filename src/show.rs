use std::fmt::Display;

use crate::maze::{Direction, Maze};

#[derive(Debug)]
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
