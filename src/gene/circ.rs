use crate::maze::circ::{CircGrid, CircMaze};

use super::{LayerMazeGenerator, Maze2dGenerator};

pub trait CircMazeGenerator {
    fn generate(&self, grid: CircGrid) -> CircMaze;
}

#[derive(Debug)]
pub struct CircMaze2dGenerator<G: Maze2dGenerator> {
    generator: G,
}

impl<G: Maze2dGenerator> CircMazeGenerator for CircMaze2dGenerator<G> {
    fn generate(&self, mut grid: CircGrid) -> CircMaze {
        self.generator.generate_2d(&mut grid);
        CircMaze::new(grid)
    }
}

impl<G: Maze2dGenerator> CircMaze2dGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
    }
}

#[derive(Debug)]
pub struct CircLayerMazeGenerator<G: LayerMazeGenerator> {
    generator: G,
}

impl<G: LayerMazeGenerator> CircMazeGenerator for CircLayerMazeGenerator<G> {
    fn generate(&self, mut grid: CircGrid) -> CircMaze {
        self.generator.generate_layer(&mut grid);
        CircMaze::new(grid)
    }
}

impl<G: LayerMazeGenerator> CircLayerMazeGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
    }
}
