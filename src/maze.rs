use std::{
    collections::{HashSet, LinkedList},
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    iter,
    path::Path,
};

use anyhow::Error as AnyError;
use image::{GenericImageView, ImageReader, Rgba};
use rand::{Rng, seq::IteratorRandom};
use thiserror::Error;

use crate::show::AsciiMazeDisplay;

#[derive(Debug, Clone, Error)]
enum Error {
    #[error(
        "Inconsistent row width in given mask, expect {expectd_width} columns, given {this_width}."
    )]
    InconsistentMaskRow {
        this_width: usize,
        expectd_width: usize,
    },
    #[error("Found isolated area in given mask, every cell in mask should be reachable.")]
    IsolatedAreaInMask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn all_dirs() -> &'static [Direction; 4] {
        static ALL_DIRECTIONS: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::West,
            Direction::South,
        ];

        &ALL_DIRECTIONS
    }

    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub r: usize,
    pub c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn random(rng: &mut impl Rng, width: usize, height: usize) -> Self {
        Self {
            r: rng.random_range(0..height),
            c: rng.random_range(0..width),
        }
    }

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::North if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::South => Some(Position::new(self.r + 1, self.c)),
            Direction::East => Some(Position::new(self.r, self.c + 1)),
            Direction::West if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            _ => None,
        }
    }

    pub fn flat_ind(&self, row_width: usize) -> usize {
        debug_assert!(self.c < row_width);
        self.r * row_width + self.c
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mask {
    width: usize,
    height: usize,
    flags: Vec<bool>,
}

impl Mask {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            flags: vec![false; width * height],
        }
    }

    pub fn try_from_text_file<P: AsRef<Path>>(path: P) -> Result<Self, AnyError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut width = None;
        let mut height = 0;
        let mut flags = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let this_width = line.chars().count();
            if *width.get_or_insert(this_width) != this_width {
                return Err(Error::InconsistentMaskRow {
                    this_width,
                    expectd_width: width.unwrap(),
                }
                .into());
            }

            flags.extend(line.chars().map(|c| !c.eq_ignore_ascii_case(&'X')));
            height += 1;
        }

        let mask = Self {
            width: width.unwrap_or(0),
            height,
            flags,
        };
        mask.check_isolation()?;
        Ok(mask)
    }

    pub fn try_from_image_file<P: AsRef<Path>>(path: P) -> Result<Self, AnyError> {
        let image = ImageReader::open(path)?.decode()?;
        let width = usize::try_from(image.width())?;
        let height = usize::try_from(image.height())?;
        let flags = image
            .pixels()
            .map(|pixel| pixel.2 != Rgba::from([0, 0, 0, 0xff]))
            .collect::<Vec<_>>();
        let mask = Self {
            width,
            height,
            flags,
        };
        mask.check_isolation()?;
        Ok(mask)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_flag(&mut self, pos: &Position, flag: bool) {
        if let Some(ind) = self.pos_to_ind(pos) {
            self.flags[ind] = flag;
        }
    }

    pub fn is_cell(&self, pos: &Position) -> bool {
        self.pos_to_ind(pos).is_some_and(|ind| self.flags[ind])
    }

    pub fn cell_pos_iter(&self) -> impl Iterator<Item = Position> {
        (0..self.flags.len())
            .filter(|ind| self.flags[*ind])
            .map(|ind| self.ind_to_pos(ind).unwrap())
    }

    pub fn cells_n(&self) -> usize {
        self.flags.iter().filter(|flag| **flag).count()
    }

    fn check_isolation(&self) -> Result<(), Error> {
        let Some(start_pos) = (0..self.height)
            .flat_map(|r| (0..self.width).map(move |c| Position::new(r, c)))
            .find(|pos| self.is_cell(pos))
        else {
            return Ok(());
        };
        let mut visited_pos = HashSet::new();
        let mut visit_list = LinkedList::from_iter(iter::once(start_pos));
        while let Some(cur_pos) = visit_list.pop_front() {
            if !visited_pos.insert(cur_pos) {
                continue;
            }

            visit_list.extend(
                Direction::all_dirs()
                    .iter()
                    .filter_map(|dir| cur_pos.neighbor(*dir))
                    .filter(|pos| self.is_cell(pos) && !visited_pos.contains(pos)),
            );
        }

        if visited_pos.len() == self.cells_n() {
            Ok(())
        } else {
            Err(Error::IsolatedAreaInMask)
        }
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.height && pos.c < self.width {
            Some(pos.flat_ind(self.width))
        } else {
            None
        }
    }

    fn ind_to_pos(&self, ind: usize) -> Option<Position> {
        if ind >= self.flags.len() {
            None
        } else {
            Some(Position::new(ind / self.width, ind % self.width))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    mask: Option<Mask>,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
            mask: None,
        }
    }

    pub fn with_mask(mask: &Mask) -> Self {
        Self {
            mask: Some(mask.clone()),
            ..Self::new(mask.width, mask.height)
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn is_cell(&self, pos: &Position) -> bool {
        if let Some(mask) = self.mask.as_ref() {
            mask.is_cell(pos)
        } else {
            self.pos_to_ind(pos).is_some()
        }
    }

    pub fn is_at_border(&self, pos: &Position, dir: Direction) -> bool {
        self.cell(pos).is_some() && self.neighbor_pos(pos, dir).is_none()
    }

    pub fn is_connect_to(&self, pos: &Position, dir: Direction) -> bool {
        if let Some(cell) = self.cell(pos) {
            self.neighbor_pos(pos, dir)
                .is_some_and(|neighbor| match dir {
                    Direction::North => cell.is_connected_to_north,
                    Direction::East => cell.is_connected_to_east,
                    other_dir => self.is_connect_to(&neighbor, other_dir.reverse()),
                })
        } else {
            false
        }
    }

    pub fn neighbor_pos(&self, pos: &Position, dir: Direction) -> Option<Position> {
        self.cell(pos).and(
            pos.neighbor(dir)
                .filter(|neighbor| self.cell(neighbor).is_some()),
        )
    }

    pub fn connect_to(&mut self, pos: &Position, dir: Direction) {
        if let Some(neighbor) = self.neighbor_pos(pos, dir) {
            if let Some(cell) = self.cell_mut(pos) {
                match dir {
                    Direction::North => cell.is_connected_to_north = true,
                    Direction::East => cell.is_connected_to_east = true,
                    other_dir => self.connect_to(&neighbor, other_dir.reverse()),
                }
            }
        }
    }

    pub fn random_cell_pos(&self, rng: &mut impl Rng) -> Option<Position> {
        if let Some(mask) = self.mask.as_ref() {
            mask.cell_pos_iter().choose(rng)
        } else {
            Some(Position::new(
                rng.random_range(0..self.height),
                rng.random_range(0..self.width),
            ))
        }
    }

    pub fn cells_n(&self) -> usize {
        if let Some(mask) = self.mask.as_ref() {
            mask.cells_n()
        } else {
            self.cells.len()
        }
    }

    fn cell(&self, pos: &Position) -> Option<&Cell> {
        if self.mask.as_ref().is_none_or(|mask| mask.is_cell(pos)) {
            self.pos_to_ind(pos).and_then(|ind| self.cells.get(ind))
        } else {
            None
        }
    }

    fn cell_mut(&mut self, pos: &Position) -> Option<&mut Cell> {
        if self.mask.as_ref().is_none_or(|mask| mask.is_cell(pos)) {
            self.pos_to_ind(pos).and_then(|ind| self.cells.get_mut(ind))
        } else {
            None
        }
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.height && pos.c < self.width {
            Some(pos.flat_ind(self.width))
        } else {
            None
        }
    }
}

impl Display for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        AsciiMazeDisplay(self).fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
struct Cell {
    is_connected_to_north: bool,
    is_connected_to_east: bool,
}

impl Cell {
    pub fn new() -> Self {
        Self::default()
    }
}
