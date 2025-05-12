use crate::maze::tri::{TriGrid, TriMaze};

use super::Maze2dGenerator;

pub trait TriMazeGenerator {
    fn generate(&self, grid: TriGrid) -> TriMaze;
}

impl<G: Maze2dGenerator> TriMazeGenerator for G {
    fn generate(&self, mut grid: TriGrid) -> TriMaze {
        self.generate_2d(&mut grid);
        TriMaze::new(grid)
    }
}
