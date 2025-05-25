use crate::maze::{
    MaskType, NoMask, WithMask,
    hexa::{HexaGrid, HexaMaze},
};

use super::{LayerMazeGenerator, Maze2dGenerator};

pub trait HexaMazeGenerator<M: MaskType> {
    fn generate(&self, grid: HexaGrid<M>) -> HexaMaze;
}

#[derive(Debug)]
pub struct HexaMaze2dGenerator<G: Maze2dGenerator> {
    generator: G,
}

impl<G: Maze2dGenerator> HexaMazeGenerator<NoMask> for HexaMaze2dGenerator<G> {
    fn generate(&self, mut grid: HexaGrid<NoMask>) -> HexaMaze {
        self.generator.generate_2d(&mut grid);
        HexaMaze::NoMask(grid)
    }
}

impl<G: Maze2dGenerator> HexaMazeGenerator<WithMask> for HexaMaze2dGenerator<G> {
    fn generate(&self, mut grid: HexaGrid<WithMask>) -> HexaMaze {
        self.generator.generate_2d(&mut grid);
        HexaMaze::WithMask(grid)
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

impl<G: LayerMazeGenerator> HexaMazeGenerator<NoMask> for HexaLayerMazeGenerator<G> {
    fn generate(&self, mut grid: HexaGrid<NoMask>) -> HexaMaze {
        self.generator.generate_layer(&mut grid);
        HexaMaze::NoMask(grid)
    }
}

impl<G: LayerMazeGenerator> HexaLayerMazeGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
    }
}
