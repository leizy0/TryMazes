use std::{collections::HashSet, fmt::Debug};

use rand::{Rng, seq::IteratorRandom};
use rect::RectMask;

pub mod circ;
pub mod hexa;
pub mod rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position2d(pub usize, pub usize);

pub trait Grid2d {
    fn cells_n(&self) -> usize;
    fn random_cell_pos(&self) -> Option<Position2d>;
    fn all_cells_pos_set(&self) -> HashSet<Position2d>;
    fn append_neighbors(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>);
    fn connect_to(&mut self, from: &Position2d, to: &Position2d) -> bool;
}

#[derive(Debug, Clone)]
pub struct GeneralRectGrid<C: Default + Debug + Clone> {
    width: usize,
    height: usize,
    cells: Vec<C>,
    mask: Option<RectMask>,
}

impl<C: Default + Debug + Clone> GeneralRectGrid<C> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![C::default(); width * height],
            mask: None,
        }
    }

    pub fn with_mask(mask: &RectMask) -> Self {
        let (width, height) = mask.size();
        Self {
            mask: Some(mask.clone()),
            ..Self::new(width, height)
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn cells_n(&self) -> usize {
        if let Some(mask) = self.mask.as_ref() {
            mask.cells_n()
        } else {
            self.cells.len()
        }
    }

    fn random_cell_pos(&self) -> Option<Position2d> {
        let mut rng = rand::rng();
        if let Some(mask) = self.mask.as_ref() {
            mask.cell_pos_iter()
                .choose(&mut rng)
                .map(|rect_pos| rect_pos.into())
        } else {
            Some(Position2d(
                rng.random_range(0..self.height),
                rng.random_range(0..self.width),
            ))
        }
    }

    fn all_cells_pos_set(&self) -> HashSet<Position2d> {
        (0..self.height)
            .flat_map(|r| (0..self.width).map(move |c| Position2d(r, c)))
            .filter(|pos| {
                self.mask
                    .as_ref()
                    .is_none_or(|mask| mask.is_cell(&(*pos).into()))
            })
            .collect()
    }

    pub fn is_cell(&self, pos: &Position2d) -> bool {
        if let Some(mask) = self.mask.as_ref() {
            mask.is_cell(&(*pos).into())
        } else {
            self.pos_to_ind(pos).is_some()
        }
    }

    fn cell(&self, pos: &Position2d) -> Option<&C> {
        if self
            .mask
            .as_ref()
            .is_none_or(|mask| mask.is_cell(&(*pos).into()))
        {
            self.pos_to_ind(pos).and_then(|ind| self.cells.get(ind))
        } else {
            None
        }
    }

    fn cell_mut(&mut self, pos: &Position2d) -> Option<&mut C> {
        if self
            .mask
            .as_ref()
            .is_none_or(|mask| mask.is_cell(&(*pos).into()))
        {
            self.pos_to_ind(pos).and_then(|ind| self.cells.get_mut(ind))
        } else {
            None
        }
    }

    fn pos_to_ind(&self, pos: &Position2d) -> Option<usize> {
        if pos.0 < self.height && pos.1 < self.width {
            Some(pos.0 * self.width + pos.1)
        } else {
            None
        }
    }
}
