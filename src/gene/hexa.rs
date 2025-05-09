use crate::maze::hexa::{HexaGrid, HexaMaze};

use super::Maze2dGenerator;

pub trait HexaMazeGenerator {
    fn generate(&self, grid: HexaGrid) -> HexaMaze;
}

impl<G: Maze2dGenerator> HexaMazeGenerator for G {
    fn generate(&self, mut grid: HexaGrid) -> HexaMaze {
        self.generate_2d(&mut grid);
        HexaMaze::new(grid)
    }
}
