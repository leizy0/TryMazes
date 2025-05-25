use crate::maze::hexa::{HexaGrid, HexaMaze};

use super::{LayerMazeGenerator, Maze2dGenerator};

pub trait HexaMazeGenerator {
    fn generate(&self, grid: HexaGrid) -> HexaMaze;
}

#[derive(Debug)]
pub struct HexaMaze2dGenerator<G: Maze2dGenerator> {
    generator: G,
}

impl<G: Maze2dGenerator> HexaMazeGenerator for HexaMaze2dGenerator<G> {
    fn generate(&self, mut grid: HexaGrid) -> HexaMaze {
        self.generator.generate_2d(&mut grid);
        HexaMaze::new(grid)
    }
}

impl<G: Maze2dGenerator> HexaMaze2dGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
    }
}

#[derive(Debug)]
pub struct HexaLayerMazeGenerator<G: LayerMazeGenerator> {
    generator: G,
}

impl<G: LayerMazeGenerator> HexaMazeGenerator for HexaLayerMazeGenerator<G> {
    fn generate(&self, mut grid: HexaGrid) -> HexaMaze {
        self.generator.generate_layer(&mut grid);
        HexaMaze::new(grid)
    }
}

impl<G: LayerMazeGenerator> HexaLayerMazeGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
    }
}
