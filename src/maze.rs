use std::collections::HashSet;

use rect::RectPosition;

pub mod rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position2d(pub usize, pub usize);

impl From<RectPosition> for Position2d {
    fn from(value: RectPosition) -> Self {
        Self(value.row, value.col)
    }
}

pub trait Grid2d {
    fn cells_n(&self) -> usize;
    fn random_cell_pos(&self) -> Option<Position2d>;
    fn all_cells_pos_set(&self) -> HashSet<Position2d>;
    fn append_neighbors(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>);
    fn connect_to(&mut self, from: &Position2d, to: &Position2d) -> bool;
}
