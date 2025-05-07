use std::{
    f32::{self, consts},
    ops::Range,
};

use rand::seq::IteratorRandom;

use super::{Grid2d, Position2d};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CircDirection {
    Inward,
    Clockwise,
    Counterclockwise,
    Outward,
}

impl CircDirection {
    pub fn all_dirs() -> &'static [Self] {
        static ALL_DIRECTIONS: [CircDirection; 4] = [
            CircDirection::Inward,
            CircDirection::Clockwise,
            CircDirection::Counterclockwise,
            CircDirection::Outward,
        ];
        &ALL_DIRECTIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CircPosition {
    pub ring: usize,
    pub cell: usize,
}

impl From<CircPosition> for Position2d {
    fn from(value: CircPosition) -> Self {
        Position2d(value.ring, value.cell)
    }
}

impl From<Position2d> for CircPosition {
    fn from(value: Position2d) -> Self {
        Self {
            ring: value.0,
            cell: value.1,
        }
    }
}

impl CircPosition {
    pub fn new(ring: usize, cell: usize) -> Self {
        Self { ring, cell }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CircCell {
    pub is_connect_inward: bool,
    pub is_connect_clockwise: bool,
}

pub enum CircCellPosIter {
    Once(Option<CircPosition>),
    CellRange {
        ring: usize,
        cell_range: Range<usize>,
        cur_cell: usize,
    },
}

impl Iterator for CircCellPosIter {
    type Item = CircPosition;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CircCellPosIter::Once(circ_position) => circ_position.take(),
            CircCellPosIter::CellRange {
                ring,
                cell_range,
                cur_cell,
            } => {
                if cell_range.contains(cur_cell) {
                    let pos = CircPosition::new(*ring, *cur_cell);
                    *cur_cell += 1;
                    Some(pos)
                } else {
                    None
                }
            }
        }
    }
}

impl CircCellPosIter {
    fn empty() -> Self {
        Self::Once(None)
    }

    fn once(ring: usize, cell: usize) -> Self {
        Self::Once(Some(CircPosition::new(ring, cell)))
    }

    fn cell_range(ring: usize, range: Range<usize>) -> Self {
        Self::CellRange {
            ring,
            cur_cell: range.start,
            cell_range: range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CircGrid {
    rings_n: usize,
    ring_end_inds: Vec<usize>,
    cells: Vec<CircCell>,
}

impl CircGrid {
    pub fn new(rings_n: usize) -> Self {
        let (ring_end_inds, cells) = Self::make_rings(rings_n);
        Self {
            rings_n,
            ring_end_inds,
            cells,
        }
    }

    fn make_rings(rings_n: usize) -> (Vec<usize>, Vec<CircCell>) {
        let mut rings_end_ind = vec![1; rings_n];
        // Calculate the number of cells in each ring.
        for ring_ind in 1..rings_n {
            let last_ring_cells_n = rings_end_ind[ring_ind - 1];
            // Make the arc length of each cell is close to ring interval(1.0 / rings_n).
            rings_end_ind[ring_ind] = last_ring_cells_n
                * (2f32 * consts::PI * ring_ind as f32 / last_ring_cells_n as f32).round() as usize;
        }
        // Accumulate the number of cells before each ring.
        for ring_ind in 1..rings_n {
            rings_end_ind[ring_ind] += rings_end_ind[ring_ind - 1];
        }

        let cells = vec![CircCell::default(); rings_end_ind.last().copied().unwrap_or(0)];
        (rings_end_ind, cells)
    }

    pub fn neighbor_pos_iter(&self, pos: &CircPosition, dir: CircDirection) -> CircCellPosIter {
        if self.pos_to_ind(pos).is_some() {
            let ring_cells_n = self.ring_cells_n(pos.ring).unwrap();
            let last_ring_cells_n = pos
                .ring
                .checked_sub(1)
                .map(|last_ring| self.ring_cells_n(last_ring).unwrap())
                .unwrap_or(0);
            match dir {
                CircDirection::Inward if pos.ring > 0 => CircCellPosIter::once(
                    pos.ring - 1,
                    if ring_cells_n > last_ring_cells_n {
                        let self_inner_ratio = ring_cells_n / last_ring_cells_n;
                        pos.cell / self_inner_ratio
                    } else {
                        debug_assert!(last_ring_cells_n == ring_cells_n);
                        pos.cell
                    },
                ),
                CircDirection::Clockwise if pos.ring > 0 => {
                    CircCellPosIter::once(pos.ring, (pos.cell + 1) % ring_cells_n)
                }
                CircDirection::Counterclockwise if pos.ring > 0 => {
                    CircCellPosIter::once(pos.ring, (pos.cell + ring_cells_n - 1) % ring_cells_n)
                }
                CircDirection::Outward => {
                    if let Some(next_ring_cells_n) = self.ring_cells_n(pos.ring + 1) {
                        let outer_self_ratio = next_ring_cells_n / ring_cells_n;
                        let cell_start = pos.cell * outer_self_ratio;
                        CircCellPosIter::cell_range(
                            pos.ring + 1,
                            cell_start..(cell_start + outer_self_ratio),
                        )
                    } else {
                        CircCellPosIter::empty()
                    }
                }
                _ => CircCellPosIter::empty(),
            }
        } else {
            CircCellPosIter::empty()
        }
    }

    fn cell_mut(&mut self, pos: &CircPosition) -> Option<&mut CircCell> {
        self.pos_to_ind(pos).and_then(|ind| self.cells.get_mut(ind))
    }

    fn cell(&self, pos: &CircPosition) -> Option<&CircCell> {
        self.pos_to_ind(pos).and_then(|ind| self.cells.get(ind))
    }

    fn ring_cells_n(&self, ring: usize) -> Option<usize> {
        self.ring_end_inds.get(ring).map(|ring_end| {
            *ring_end
                - ring
                    .checked_sub(1)
                    .map(|last_ring| self.ring_end_inds[last_ring])
                    .unwrap_or(0)
        })
    }

    fn pos_to_ind(&self, pos: &CircPosition) -> Option<usize> {
        if pos.ring >= self.rings_n {
            return None;
        }

        let ring_start = pos
            .ring
            .checked_sub(1)
            .and_then(|last_ring| self.ring_end_inds.get(last_ring).copied())
            .unwrap_or(0);
        Some(ring_start + pos.cell).filter(|ind| *ind < self.ring_end_inds[pos.ring])
    }

    fn ind_to_pos(&self, ind: usize) -> Option<CircPosition> {
        if ind >= self.cells.len() {
            return None;
        }

        Some(match self.ring_end_inds.binary_search(&ind) {
            Ok(last_ring) => CircPosition::new(last_ring + 1, ind - self.ring_end_inds[last_ring]),
            Err(ring) => CircPosition::new(
                ring,
                ind - ring
                    .checked_sub(1)
                    .map(|last_ring| self.ring_end_inds[last_ring])
                    .unwrap_or(0),
            ),
        })
    }
}

impl Grid2d for CircGrid {
    fn cells_n(&self) -> usize {
        self.ring_end_inds.last().copied().unwrap_or(0)
    }

    fn random_cell_pos(&self) -> Option<super::Position2d> {
        let mut rng = rand::rng();
        (0..self.cells_n())
            .choose(&mut rng)
            .and_then(|cell_ind| self.ind_to_pos(cell_ind))
            .map(Position2d::from)
    }

    fn all_cells_pos_set(&self) -> std::collections::HashSet<super::Position2d> {
        (0..self.cells_n())
            .map(|ind| self.ind_to_pos(ind).unwrap().into())
            .collect()
    }

    fn append_neighbors(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        let pos = (*pos).into();
        neighbors.extend(
            CircDirection::all_dirs()
                .iter()
                .flat_map(|dir| self.neighbor_pos_iter(&pos, *dir))
                .map(Position2d::from),
        );
    }

    fn connect_to(&mut self, from: &Position2d, to: &Position2d) -> bool {
        let from = (*from).into();
        let to = (*to).into();
        let Some(dir) = CircDirection::all_dirs()
            .iter()
            .find(|dir| {
                self.neighbor_pos_iter(&from, **dir)
                    .any(|neighbor| neighbor == to)
            })
            .copied()
        else {
            return false;
        };
        match dir {
            CircDirection::Inward => self.cell_mut(&from).unwrap().is_connect_inward = true,
            CircDirection::Clockwise => self.cell_mut(&from).unwrap().is_connect_clockwise = true,
            CircDirection::Counterclockwise => {
                self.cell_mut(&to).unwrap().is_connect_clockwise = true
            }
            CircDirection::Outward => self.cell_mut(&to).unwrap().is_connect_inward = true,
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct CircMaze {
    grid: CircGrid,
}

impl CircMaze {
    pub fn new(grid: CircGrid) -> Self {
        Self { grid }
    }

    pub fn rings_n(&self) -> usize {
        self.grid.rings_n
    }

    pub fn ring_cells_n(&self, ring: usize) -> Option<usize> {
        self.grid.ring_cells_n(ring)
    }

    pub fn is_connect_inward(&self, pos: &CircPosition) -> bool {
        self.grid
            .cell(pos)
            .is_some_and(|cell| cell.is_connect_inward)
    }

    pub fn is_connect_clockwise(&self, pos: &CircPosition) -> bool {
        self.grid
            .cell(pos)
            .is_some_and(|cell| cell.is_connect_clockwise)
    }
}

#[cfg(test)]
mod test {
    use crate::maze::circ::CircPosition;

    use super::{CircDirection, CircGrid};

    #[test]
    fn test_grid_make_rings() {
        let (rings_end_ind, cells) = CircGrid::make_rings(7);
        assert_eq!(rings_end_ind.as_slice(), &[1, 7, 19, 43, 67, 91, 139]);
        assert_eq!(cells.len(), 139);
    }

    #[test]
    fn test_grid_ring_cells_n() {
        let grid = CircGrid::new(7);
        assert_eq!(grid.ring_cells_n(0), Some(1));
        assert_eq!(grid.ring_cells_n(1), Some(6));
        assert_eq!(grid.ring_cells_n(2), Some(12));
        assert_eq!(grid.ring_cells_n(3), Some(24));
        assert_eq!(grid.ring_cells_n(4), Some(24));
        assert_eq!(grid.ring_cells_n(5), Some(24));
        assert_eq!(grid.ring_cells_n(6), Some(48));
        assert_eq!(grid.ring_cells_n(7), None);
    }

    #[test]
    fn test_grid_pos_to_ind() {
        let grid = CircGrid::new(7);
        assert_eq!(grid.pos_to_ind(&CircPosition::new(0, 0)), Some(0));
        assert_eq!(grid.pos_to_ind(&CircPosition::new(0, 1)), None);
        assert_eq!(grid.pos_to_ind(&CircPosition::new(1, 4)), Some(5));
        assert_eq!(grid.pos_to_ind(&CircPosition::new(2, 4)), Some(11));
        assert_eq!(grid.pos_to_ind(&CircPosition::new(5, 8)), Some(75));
        assert_eq!(grid.pos_to_ind(&CircPosition::new(7, 0)), None);
        assert_eq!(grid.pos_to_ind(&CircPosition::new(7, 4)), None);
    }

    #[test]
    fn test_grid_ind_to_pos() {
        let grid = CircGrid::new(7);
        assert_eq!(grid.ind_to_pos(0), Some(CircPosition::new(0, 0)));
        assert_eq!(grid.ind_to_pos(1), Some(CircPosition::new(1, 0)));
        assert_eq!(grid.ind_to_pos(28), Some(CircPosition::new(3, 9)));
        assert_eq!(grid.ind_to_pos(138), Some(CircPosition::new(6, 47)));
        assert_eq!(grid.ind_to_pos(139), None);
    }

    #[test]
    fn test_grid_neighbor_pos_iter() {
        let grid = CircGrid::new(7);
        let center_cell_pos = CircPosition::new(0, 0);
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&center_cell_pos, CircDirection::Inward),
            &[],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&center_cell_pos, CircDirection::Clockwise),
            &[],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&center_cell_pos, CircDirection::Counterclockwise),
            &[],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&center_cell_pos, CircDirection::Outward),
            &[(1, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)],
        );

        let another_cell_pos = CircPosition::new(2, 11);
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&another_cell_pos, CircDirection::Inward),
            &[(1, 5)],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&another_cell_pos, CircDirection::Clockwise),
            &[(2, 0)],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&another_cell_pos, CircDirection::Counterclockwise),
            &[(2, 10)],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&another_cell_pos, CircDirection::Outward),
            &[(3, 22), (3, 23)],
        );

        let bound_cell_pos = CircPosition::new(6, 8);
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&bound_cell_pos, CircDirection::Inward),
            &[(5, 4)],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&bound_cell_pos, CircDirection::Clockwise),
            &[(6, 9)],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&bound_cell_pos, CircDirection::Counterclockwise),
            &[(6, 7)],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&bound_cell_pos, CircDirection::Outward),
            &[],
        );

        let not_cell_pos = CircPosition::new(7, 8);
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&not_cell_pos, CircDirection::Inward),
            &[],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&not_cell_pos, CircDirection::Clockwise),
            &[],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&not_cell_pos, CircDirection::Counterclockwise),
            &[],
        );
        assert_pos_iter_eq(
            grid.neighbor_pos_iter(&not_cell_pos, CircDirection::Outward),
            &[],
        );
    }

    fn assert_pos_iter_eq(
        pos_iter: impl Iterator<Item = CircPosition>,
        expect_pos: &[(usize, usize)],
    ) {
        let mut pos_ind = 0;
        for actual in pos_iter {
            assert!(pos_ind < expect_pos.len());
            assert_eq!(
                actual,
                CircPosition::new(expect_pos[pos_ind].0, expect_pos[pos_ind].1)
            );
            pos_ind += 1;
        }

        assert_eq!(pos_ind, expect_pos.len());
    }
}
