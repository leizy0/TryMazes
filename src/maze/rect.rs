use std::{
    collections::{HashSet, LinkedList},
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    iter,
    marker::PhantomData,
    path::Path,
};

use anyhow::Error as AnyError;
use image::{GenericImageView, ImageReader, Rgba};
use rand::Rng;
use thiserror::Error;

use crate::show::rect::{AsciiBoxCharset, RectMazeCmdDisplay};

use super::{GeneralRectGrid, Grid2d, LayerGrid, MaskType, NoMask, Position2d, WithMask};

#[derive(Debug, Clone, Error)]
enum Error {
    #[error(
        "Inconsistent row width in given rectangular mask, expect {expectd_width} columns, given {this_width}."
    )]
    InconsistentRectMaskRow {
        this_width: usize,
        expectd_width: usize,
    },
    #[error(
        "Found isolated area in given rectangular mask, every cell in mask should be reachable."
    )]
    IsolatedAreaInRectMask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RectDirection {
    North,
    South,
    East,
    West,
}

impl RectDirection {
    pub fn all_dirs() -> &'static [RectDirection; 4] {
        static ALL_DIRECTIONS: [RectDirection; 4] = [
            RectDirection::North,
            RectDirection::East,
            RectDirection::West,
            RectDirection::South,
        ];

        &ALL_DIRECTIONS
    }

    pub fn reverse(&self) -> Self {
        match self {
            RectDirection::North => RectDirection::South,
            RectDirection::South => RectDirection::North,
            RectDirection::East => RectDirection::West,
            RectDirection::West => RectDirection::East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RectPosition {
    pub row: usize,
    pub col: usize,
}

impl From<RectPosition> for Position2d {
    fn from(value: RectPosition) -> Self {
        Self(value.row, value.col)
    }
}

impl From<Position2d> for RectPosition {
    fn from(value: Position2d) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

impl RectPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn random(rng: &mut impl Rng, width: usize, height: usize) -> Self {
        Self {
            row: rng.random_range(0..height),
            col: rng.random_range(0..width),
        }
    }

    pub fn neighbor(&self, dir: RectDirection) -> Option<Self> {
        match dir {
            RectDirection::North if self.row > 0 => Some(RectPosition::new(self.row - 1, self.col)),
            RectDirection::South => Some(RectPosition::new(self.row + 1, self.col)),
            RectDirection::East => Some(RectPosition::new(self.row, self.col + 1)),
            RectDirection::West if self.col > 0 => Some(RectPosition::new(self.row, self.col - 1)),
            _ => None,
        }
    }

    pub fn flat_ind(&self, row_width: usize) -> usize {
        debug_assert!(self.col < row_width);
        self.row * row_width + self.col
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RectMask {
    width: usize,
    height: usize,
    flags: Vec<bool>,
}

impl RectMask {
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
                return Err(Error::InconsistentRectMaskRow {
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

    pub fn set_flag(&mut self, pos: &RectPosition, flag: bool) {
        if let Some(ind) = self.pos_to_ind(pos) {
            self.flags[ind] = flag;
        }
    }

    pub fn is_cell(&self, pos: &RectPosition) -> bool {
        self.pos_to_ind(pos).is_some_and(|ind| self.flags[ind])
    }

    pub fn cell_pos_iter(&self) -> impl Iterator<Item = RectPosition> {
        (0..self.flags.len())
            .filter(|ind| self.flags[*ind])
            .map(|ind| self.ind_to_pos(ind).unwrap())
    }

    pub fn cells_n(&self) -> usize {
        self.flags.iter().filter(|flag| **flag).count()
    }

    fn check_isolation(&self) -> Result<(), Error> {
        let Some(start_pos) = (0..self.height)
            .flat_map(|r| (0..self.width).map(move |c| RectPosition::new(r, c)))
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
                RectDirection::all_dirs()
                    .iter()
                    .filter_map(|dir| cur_pos.neighbor(*dir))
                    .filter(|pos| self.is_cell(pos) && !visited_pos.contains(pos)),
            );
        }

        if visited_pos.len() == self.cells_n() {
            Ok(())
        } else {
            Err(Error::IsolatedAreaInRectMask)
        }
    }

    fn pos_to_ind(&self, pos: &RectPosition) -> Option<usize> {
        if pos.row < self.height && pos.col < self.width {
            Some(pos.flat_ind(self.width))
        } else {
            None
        }
    }

    fn ind_to_pos(&self, ind: usize) -> Option<RectPosition> {
        if ind >= self.flags.len() {
            None
        } else {
            Some(RectPosition::new(ind / self.width, ind % self.width))
        }
    }
}

#[derive(Debug, Clone, Default)]
struct RectCell {
    is_connected_to_north: bool,
    is_connected_to_east: bool,
}

#[derive(Debug, Clone)]
pub struct RectGrid<M: MaskType>(GeneralRectGrid<RectCell>, PhantomData<M>);

impl<M: MaskType + Clone> Grid2d for RectGrid<M> {
    fn cells_n(&self) -> usize {
        self.0.cells_n()
    }

    fn random_cell_pos(&self) -> Option<Position2d> {
        self.0.random_cell_pos()
    }

    fn all_cells_pos_set(&self) -> HashSet<Position2d> {
        self.0.all_cells_pos_set()
    }

    fn append_neighbors(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        let rect_pos: RectPosition = (*pos).into();
        if self.0.is_cell(pos) {
            neighbors.extend(
                RectDirection::all_dirs()
                    .iter()
                    .filter_map(|dir| {
                        rect_pos
                            .neighbor(*dir)
                            .filter(|neighbor| self.0.is_cell(&(*neighbor).into()))
                    })
                    .map(Position2d::from),
            );
        }
    }

    fn connect_to(&mut self, from: &Position2d, to: &Position2d) -> bool {
        let rect_from: RectPosition = (*from).into();
        let rect_to: RectPosition = (*to).into();
        self.0.is_cell(from)
            && self.0.is_cell(to)
            && RectDirection::all_dirs()
                .iter()
                .find(|dir| {
                    rect_from
                        .neighbor(**dir)
                        .is_some_and(|neighbor| neighbor == rect_to)
                })
                .is_some_and(|connect_dir| self.connect_to(&rect_from, *connect_dir))
    }
}

impl LayerGrid for RectGrid<NoMask> {
    fn layers_n(&self) -> usize {
        self.0.height
    }

    fn cells_n_at(&self, _layer_ind: usize) -> usize {
        self.0.width
    }

    fn append_neighbors_upper_layer(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        neighbors.extend(
            self.neighbor_pos(&(*pos).into(), RectDirection::North)
                .map(Position2d::from),
        );
    }

    fn append_neighbors_lower_layer(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        neighbors.extend(
            self.neighbor_pos(&(*pos).into(), RectDirection::South)
                .map(Position2d::from),
        );
    }
}

impl RectGrid<NoMask> {
    pub fn new(width: usize, height: usize) -> Self {
        RectGrid::<NoMask>(
            GeneralRectGrid::<RectCell>::new(width, height),
            PhantomData::<NoMask>,
        )
    }
}

impl RectGrid<WithMask> {
    pub fn new(mask: &RectMask) -> Self {
        RectGrid::<WithMask>(
            GeneralRectGrid::<RectCell>::with_mask(mask),
            PhantomData::<WithMask>,
        )
    }
}

impl<M: MaskType> RectGrid<M> {
    pub fn size(&self) -> (usize, usize) {
        self.0.size()
    }

    pub fn is_at_border(&self, pos: &RectPosition, dir: RectDirection) -> bool {
        self.0.cell(&(*pos).into()).is_some() && self.neighbor_pos(pos, dir).is_none()
    }

    pub fn is_connected_to(&self, pos: &RectPosition, dir: RectDirection) -> bool {
        if let Some(cell) = self.0.cell(&(*pos).into()) {
            self.neighbor_pos(pos, dir)
                .is_some_and(|neighbor| match dir {
                    RectDirection::North => cell.is_connected_to_north,
                    RectDirection::East => cell.is_connected_to_east,
                    other_dir => self.is_connected_to(&neighbor, other_dir.reverse()),
                })
        } else {
            false
        }
    }

    pub fn neighbor_pos(&self, pos: &RectPosition, dir: RectDirection) -> Option<RectPosition> {
        self.0.cell(&(*pos).into()).and(
            pos.neighbor(dir)
                .filter(|neighbor| self.0.is_cell(&(*neighbor).into())),
        )
    }

    pub fn connect_to(&mut self, pos: &RectPosition, dir: RectDirection) -> bool {
        if let Some(neighbor) = self.neighbor_pos(pos, dir) {
            if let Some(cell) = self.0.cell_mut(&(*pos).into()) {
                return match dir {
                    RectDirection::North => {
                        cell.is_connected_to_north = true;
                        true
                    }
                    RectDirection::East => {
                        cell.is_connected_to_east = true;
                        true
                    }
                    other_dir => self.connect_to(&neighbor, other_dir.reverse()),
                };
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub enum RectMaze {
    NoMask(RectGrid<NoMask>),
    WithMask(RectGrid<WithMask>),
}

impl Display for RectMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RectMazeCmdDisplay(self, AsciiBoxCharset).fmt(f)
    }
}

impl RectMaze {
    pub fn size(&self) -> (usize, usize) {
        match self {
            RectMaze::NoMask(rect_grid) => rect_grid.size(),
            RectMaze::WithMask(rect_grid) => rect_grid.size(),
        }
    }

    pub fn is_cell(&self, pos: &RectPosition) -> bool {
        match self {
            RectMaze::NoMask(rect_grid) => rect_grid.0.is_cell(&(*pos).into()),
            RectMaze::WithMask(rect_grid) => rect_grid.0.is_cell(&(*pos).into()),
        }
    }

    pub fn is_connected_to(&self, pos: &RectPosition, dir: RectDirection) -> bool {
        match self {
            RectMaze::NoMask(rect_grid) => rect_grid.is_connected_to(pos, dir),
            RectMaze::WithMask(rect_grid) => rect_grid.is_connected_to(pos, dir),
        }
    }
}
