use std::{
    collections::{HashMap, HashSet},
    iter,
};

use clap::ValueEnum;
use rand::{Rng, seq::IteratorRandom};

use crate::maze::{Direction, Mask, Maze, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash)]
pub enum DiagonalDirection {
    Northeast,
    Southeast,
    Southwest,
    Northwest,
}

impl DiagonalDirection {
    pub fn hv_dirs(&self) -> (Direction, Direction) {
        match self {
            DiagonalDirection::Northeast => (Direction::East, Direction::North),
            DiagonalDirection::Southeast => (Direction::East, Direction::South),
            DiagonalDirection::Southwest => (Direction::West, Direction::South),
            DiagonalDirection::Northwest => (Direction::West, Direction::North),
        }
    }
}

pub trait MazeGenerator {
    fn generate(&self) -> Maze;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BTreeMazeGenerator {
    width: usize,
    height: usize,
    con_dir: DiagonalDirection,
}

impl BTreeMazeGenerator {
    pub fn new(width: usize, height: usize, con_dir: DiagonalDirection) -> Self {
        Self {
            width,
            height,
            con_dir,
        }
    }
}

impl MazeGenerator for BTreeMazeGenerator {
    fn generate(&self) -> Maze {
        let mut maze = Maze::new(self.width, self.height);
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let connect_dirs = [horz_dir, vert_dir];
        for r_ind in 0..self.height {
            for c_ind in 0..self.width {
                let pos = Position::new(r_ind, c_ind);
                let at_horz_border = maze.is_at_border(&pos, horz_dir);
                let at_vert_border = maze.is_at_border(&pos, vert_dir);

                if at_horz_border {
                    if !at_vert_border {
                        maze.connect_to(&pos, vert_dir);
                    }
                } else if at_vert_border {
                    maze.connect_to(&pos, horz_dir);
                } else {
                    let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                    maze.connect_to(&pos, rand_dir);
                }
            }
        }

        maze
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SideWinderMazeGenerator {
    width: usize,
    height: usize,
    con_dir: DiagonalDirection,
}

impl SideWinderMazeGenerator {
    pub fn new(width: usize, height: usize, con_dir: DiagonalDirection) -> Self {
        Self {
            width,
            height,
            con_dir,
        }
    }
}

impl MazeGenerator for SideWinderMazeGenerator {
    fn generate(&self) -> Maze {
        let mut maze = Maze::new(self.width, self.height);
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let is_horz_reverse = horz_dir == Direction::West;
        for r_ind in 0..self.height {
            let mut run_start_ind = if is_horz_reverse { self.width - 1 } else { 0 };
            for c_ind in 0..self.width {
                let c_ind = if is_horz_reverse {
                    self.width - 1 - c_ind
                } else {
                    c_ind
                };
                let pos = Position::new(r_ind, c_ind);
                let at_horz_border = maze.is_at_border(&pos, horz_dir);
                let at_vert_border = maze.is_at_border(&pos, vert_dir);
                let close_out = !at_vert_border && (at_horz_border || rng.random::<bool>());

                if close_out {
                    let out_ind = if is_horz_reverse {
                        rng.random_range(c_ind..=run_start_ind)
                    } else {
                        rng.random_range(run_start_ind..=c_ind)
                    };
                    maze.connect_to(&Position::new(r_ind, out_ind), vert_dir);
                    run_start_ind = if is_horz_reverse {
                        c_ind.saturating_sub(1)
                    } else {
                        c_ind + 1
                    };
                } else if !at_horz_border {
                    maze.connect_to(&pos, horz_dir);
                }
            }
        }

        maze
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AldousBroderMazeGenerator {
    width: usize,
    height: usize,
    mask: Option<Mask>,
}

impl MazeGenerator for AldousBroderMazeGenerator {
    fn generate(&self) -> Maze {
        let mut maze = if let Some(mask) = self.mask.as_ref() {
            Maze::with_mask(mask)
        } else {
            Maze::new(self.width, self.height)
        };

        let mut visited_marks = vec![false; self.width * self.height];
        let mut candidate_neighbors = [(Direction::North, Position::new(0, 0)); 4];
        let mut rng = rand::rng();
        let Some(mut cur_pos) = maze.random_cell_pos(&mut rng) else {
            return maze;
        };
        visited_marks[cur_pos.flat_ind(self.width)] = true;
        let mut unvisited_cells_n = maze.cells_n() - 1;
        while unvisited_cells_n > 0 {
            let mut candidates_n = 0;
            for candidate in Direction::all_dirs()
                .iter()
                .filter_map(|dir| maze.neighbor_pos(&cur_pos, *dir).map(|pos| (*dir, pos)))
            {
                candidate_neighbors[candidates_n] = candidate;
                candidates_n += 1;
            }

            let candidate_ind = rng.random_range(0..candidates_n);
            let (candidate_dir, candidate_pos) = candidate_neighbors[candidate_ind];
            if !visited_marks[candidate_pos.flat_ind(self.width)] {
                maze.connect_to(&cur_pos, candidate_dir);
                visited_marks[candidate_pos.flat_ind(self.width)] = true;
                unvisited_cells_n -= 1;
            }

            cur_pos = candidate_pos;
        }

        maze
    }
}

impl AldousBroderMazeGenerator {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            mask: None,
        }
    }

    pub fn with_mask(mask: Mask) -> Self {
        let (width, height) = mask.size();
        Self {
            width,
            height,
            mask: Some(mask),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct WilsonMazeGenerator {
    width: usize,
    height: usize,
    mask: Option<Mask>,
}

impl MazeGenerator for WilsonMazeGenerator {
    fn generate(&self) -> Maze {
        let (mut maze, mut unvisited_pos) = if let Some(mask) = self.mask.as_ref() {
            (
                Maze::with_mask(mask),
                mask.cell_pos_iter().collect::<HashSet<_>>(),
            )
        } else {
            (
                Maze::new(self.width, self.height),
                (0..self.height)
                    .flat_map(|r| (0..self.width).map(move |c| Position::new(r, c)))
                    .collect::<HashSet<_>>(),
            )
        };

        let mut rng = rand::rng();
        let Some(first_visited_pos) = unvisited_pos.iter().choose(&mut rng).copied() else {
            return maze;
        };
        unvisited_pos.remove(&first_visited_pos);
        let mut cur_path = Vec::new();
        let mut candidate_neighbors = [(Direction::North, Position::new(0, 0)); 4];
        let mut walk_visited_pos = HashMap::new();
        while !unvisited_pos.is_empty() {
            cur_path.clear();
            walk_visited_pos.clear();
            let Some(mut cur_pos) = unvisited_pos.iter().choose(&mut rng).copied() else {
                break;
            };
            loop {
                // Random walk.
                let mut candidates_n = 0;
                for candidate in Direction::all_dirs().iter().filter_map(|dir| {
                    maze.neighbor_pos(&cur_pos, *dir)
                        .map(|neighbor| (*dir, neighbor))
                }) {
                    candidate_neighbors[candidates_n] = candidate;
                    candidates_n += 1;
                }
                let candidate_ind = rng.random_range(0..candidates_n);
                let (candidate_dir, candidate_pos) = candidate_neighbors[candidate_ind];
                walk_visited_pos.insert(cur_pos, cur_path.len());
                cur_path.push((cur_pos, candidate_dir));
                if !unvisited_pos.contains(&candidate_pos) {
                    // Touch visited position, path ends.
                    break;
                }

                if let Some(path_ind) = walk_visited_pos.get(&candidate_pos).copied() {
                    // Remove loop.
                    for _ in path_ind..cur_path.len() {
                        let (pos, _) = cur_path.pop().unwrap();
                        walk_visited_pos.remove(&pos);
                    }
                }
                cur_pos = candidate_pos;
            }

            for (pos, dir) in cur_path.iter() {
                maze.connect_to(pos, *dir);
                debug_assert!(unvisited_pos.contains(pos));
                unvisited_pos.remove(pos);
            }
        }

        maze
    }
}

impl WilsonMazeGenerator {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            mask: None,
        }
    }

    pub fn with_mask(mask: Mask) -> Self {
        let (width, height) = mask.size();
        Self {
            width,
            height,
            mask: Some(mask),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HuntAndKillMazeGenerator {
    width: usize,
    height: usize,
    mask: Option<Mask>,
}

impl MazeGenerator for HuntAndKillMazeGenerator {
    fn generate(&self) -> Maze {
        let mut maze = if let Some(mask) = self.mask.as_ref() {
            Maze::with_mask(mask)
        } else {
            Maze::new(self.width, self.height)
        };

        let mut visited_marks = vec![false; self.width * self.height];
        let mut rng = rand::rng();
        let Some(mut cur_pos) = maze.random_cell_pos(&mut rng) else {
            return maze;
        };
        let mut unvisited_cells_n = maze.cells_n();
        let mut candidate_neighbors = [(Direction::North, Position::new(0, 0)); 4];
        while unvisited_cells_n > 0 {
            // Kill phase.
            let mut candidates_n = 0;
            loop {
                visited_marks[cur_pos.flat_ind(self.width)] = true;
                unvisited_cells_n -= 1;
                // Select an unvisited candidate randomly.
                candidates_n = 0;
                for candidate in Direction::all_dirs().iter().filter_map(|dir| {
                    maze.neighbor_pos(&cur_pos, *dir)
                        .filter(|neighbor| !visited_marks[neighbor.flat_ind(self.width)])
                        .map(|neighbor| (*dir, neighbor))
                }) {
                    candidate_neighbors[candidates_n] = candidate;
                    candidates_n += 1;
                }

                if candidates_n == 0 {
                    break;
                }

                let candidate_ind = rng.random_range(0..candidates_n);
                let (candidate_dir, candidate_pos) = candidate_neighbors[candidate_ind];
                maze.connect_to(&cur_pos, candidate_dir);
                cur_pos = candidate_pos;
            }

            if unvisited_cells_n == 0 {
                break;
            }

            // Hunt phase.
            'hunt: for r_ind in 0..self.height {
                for c_ind in 0..self.width {
                    let hunt_pos = Position::new(r_ind, c_ind);
                    if !visited_marks[hunt_pos.flat_ind(self.width)] {
                        // Select a visited candidate randomly.
                        candidates_n = 0;
                        for candidate in Direction::all_dirs().iter().filter_map(|dir| {
                            maze.neighbor_pos(&hunt_pos, *dir)
                                .filter(|neighbor| visited_marks[neighbor.flat_ind(self.width)])
                                .map(|neighbor| (*dir, neighbor))
                        }) {
                            candidate_neighbors[candidates_n] = candidate;
                            candidates_n += 1;
                        }

                        if candidates_n == 0 {
                            continue;
                        }

                        let candidate_ind = rng.random_range(0..candidates_n);
                        let (candidate_dir, candidate_pos) = candidate_neighbors[candidate_ind];
                        maze.connect_to(&candidate_pos, candidate_dir.reverse());
                        cur_pos = hunt_pos;
                        break 'hunt;
                    }
                }
            }
        }

        maze
    }
}

impl HuntAndKillMazeGenerator {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            mask: None,
        }
    }

    pub fn with_mask(mask: Mask) -> Self {
        let (width, height) = mask.size();
        Self {
            width,
            height,
            mask: Some(mask),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RecursiveBacktrackerMazeGenerator {
    width: usize,
    height: usize,
    mask: Option<Mask>,
}

impl MazeGenerator for RecursiveBacktrackerMazeGenerator {
    fn generate(&self) -> Maze {
        let mut maze = if let Some(mask) = self.mask.as_ref() {
            Maze::with_mask(mask)
        } else {
            Maze::new(self.width, self.height)
        };

        let mut rng = rand::rng();
        let Some(start_pos) = maze.random_cell_pos(&mut rng) else {
            return maze;
        };
        let mut visited_marks = vec![false; self.width * self.height];
        let mut unvisited_cells_n = maze.cells_n();
        let mut candidate_neighbors = [(Direction::North, Position::new(0, 0)); 4];
        let mut visited_stack = Vec::from_iter(iter::once(start_pos));
        while unvisited_cells_n > 0 {
            let Some(cur_pos) = visited_stack.last().copied() else {
                break;
            };
            if !visited_marks[cur_pos.flat_ind(self.width)] {
                visited_marks[cur_pos.flat_ind(self.width)] = true;
                unvisited_cells_n -= 1;
            }

            // Select an unvisited candidate randomly.
            let mut candidates_n = 0;
            for candidate in Direction::all_dirs().iter().filter_map(|dir| {
                maze.neighbor_pos(&cur_pos, *dir)
                    .filter(|neighbor| !visited_marks[neighbor.flat_ind(self.width)])
                    .map(|neighbor| (*dir, neighbor))
            }) {
                candidate_neighbors[candidates_n] = candidate;
                candidates_n += 1;
            }

            if candidates_n == 0 {
                // At dead end, try to backtrack.
                visited_stack.pop();
                continue;
            }

            let candidate_ind = rng.random_range(0..candidates_n);
            let (candidate_dir, candidate_pos) = candidate_neighbors[candidate_ind];
            maze.connect_to(&cur_pos, candidate_dir);
            visited_stack.push(candidate_pos);
        }

        maze
    }
}

impl RecursiveBacktrackerMazeGenerator {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            mask: None,
        }
    }

    pub fn with_mask(mask: Mask) -> Self {
        let (width, height) = mask.size();
        Self {
            width,
            height,
            mask: Some(mask),
        }
    }
}
