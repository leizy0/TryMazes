use crate::maze::circ::{CircGrid, CircMaze};

use super::Maze2dGenerator;

pub trait CircMazeGenerator {
    fn generate(&self, grid: CircGrid) -> CircMaze;
}

impl<T: Maze2dGenerator> CircMazeGenerator for T {
    fn generate(&self, mut grid: CircGrid) -> CircMaze {
        self.generate_2d(&mut grid);
        CircMaze::new(grid)
    }
}
