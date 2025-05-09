use std::{
    collections::{HashMap, HashSet},
    iter,
};

use rand::seq::IteratorRandom;

use crate::maze::Grid2d;

pub mod circ;
pub mod hexa;
pub mod rect;

pub trait Maze2dGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AldousBroderMazeGenerator;

impl Maze2dGenerator for AldousBroderMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut visited_pos = HashSet::new();
        let mut rng = rand::rng();
        let Some(mut cur_pos) = grid.random_cell_pos() else {
            // Empty grid
            return;
        };
        visited_pos.insert(cur_pos);
        let mut unvisited_cells_n = grid.cells_n() - 1;
        let mut neighbors = Vec::new();
        while unvisited_cells_n > 0 {
            neighbors.clear();
            grid.append_neighbors(&cur_pos, &mut neighbors);
            let candidate = neighbors
                .iter()
                .choose(&mut rng)
                .expect("There should be at least one neighbor in given non-empty grid.");
            if !visited_pos.contains(candidate) {
                grid.connect_to(&cur_pos, candidate);
                visited_pos.insert(*candidate);
                unvisited_cells_n -= 1;
            }

            cur_pos = *candidate;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct WilsonMazeGenerator;

impl Maze2dGenerator for WilsonMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut unvisited_pos = grid.all_cells_pos_set();
        let mut rng = rand::rng();
        let Some(first_visited_pos) = unvisited_pos.iter().choose(&mut rng).copied() else {
            return;
        };
        unvisited_pos.remove(&first_visited_pos);
        let mut cur_path = Vec::new();
        let mut walk_visited_pos = HashMap::new();
        let mut neighbors = Vec::new();
        while !unvisited_pos.is_empty() {
            cur_path.clear();
            walk_visited_pos.clear();
            let Some(mut cur_pos) = unvisited_pos.iter().choose(&mut rng).copied() else {
                break;
            };
            loop {
                // Random walk.
                neighbors.clear();
                grid.append_neighbors(&cur_pos, &mut neighbors);
                let candidate = neighbors
                    .iter()
                    .choose(&mut rng)
                    .expect("There should be at least one neighbor in given non-empty grid.");
                walk_visited_pos.insert(cur_pos, cur_path.len());
                cur_path.push(cur_pos);
                if !unvisited_pos.contains(candidate) {
                    // Touch visited position in previous walks, path ends.
                    cur_path.push(*candidate);
                    break;
                }

                if let Some(path_ind) = walk_visited_pos.get(candidate).copied() {
                    // Remove loop.
                    for _ in path_ind..cur_path.len() {
                        let remove_pos = cur_path.pop().unwrap();
                        walk_visited_pos.remove(&remove_pos);
                    }
                }
                cur_pos = *candidate;
            }

            debug_assert!(cur_path.len() >= 2);
            for (from, to) in cur_path[0..cur_path.len().saturating_sub(1)]
                .iter()
                .zip(cur_path[1..].iter())
            {
                grid.connect_to(from, to);
                debug_assert!(unvisited_pos.contains(from));
                unvisited_pos.remove(from);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HuntAndKillMazeGenerator;

impl Maze2dGenerator for HuntAndKillMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut unvisited_pos = grid.all_cells_pos_set();
        let mut rng = rand::rng();
        let Some(mut cur_pos) = grid.random_cell_pos() else {
            return;
        };
        let mut neighbors = Vec::new();
        while !unvisited_pos.is_empty() {
            loop {
                unvisited_pos.remove(&cur_pos);
                // Select an unvisited candidate randomly.
                neighbors.clear();
                grid.append_neighbors(&cur_pos, &mut neighbors);
                let Some(candidate) = neighbors
                    .iter()
                    .filter(|neighbor| unvisited_pos.contains(neighbor))
                    .choose(&mut rng)
                else {
                    break;
                };
                grid.connect_to(&cur_pos, candidate);
                cur_pos = *candidate;
            }

            // // Hunt phase.
            let Some((next_start_pos, prev_visited_pos)) = unvisited_pos
                .iter()
                .filter_map(|pos| {
                    neighbors.clear();
                    grid.append_neighbors(pos, &mut neighbors);
                    neighbors
                        .iter()
                        .filter(|neighbor| !unvisited_pos.contains(neighbor))
                        .choose(&mut rng)
                        .map(|neighbor| (*pos, *neighbor))
                })
                .next()
            else {
                break;
            };
            grid.connect_to(&prev_visited_pos, &next_start_pos);
            cur_pos = next_start_pos;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RecursiveBacktrackerMazeGenerator;

impl Maze2dGenerator for RecursiveBacktrackerMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut rng = rand::rng();
        let Some(start_pos) = grid.random_cell_pos() else {
            return;
        };
        let mut visited_pos = HashSet::new();
        let mut unvisited_cells_n = grid.cells_n();
        let mut visited_stack = Vec::from_iter(iter::once(start_pos));
        let mut neighbors = Vec::new();
        while unvisited_cells_n > 0 {
            let Some(cur_pos) = visited_stack.last().copied() else {
                break;
            };
            if visited_pos.insert(cur_pos) {
                unvisited_cells_n -= 1;
            }

            // Select an unvisited candidate randomly.
            neighbors.clear();
            grid.append_neighbors(&cur_pos, &mut neighbors);
            let Some(candidate) = neighbors
                .iter()
                .filter(|neighbor| !visited_pos.contains(neighbor))
                .choose(&mut rng)
            else {
                // At dead end, try to backtrack.
                visited_stack.pop();
                continue;
            };
            grid.connect_to(&cur_pos, candidate);
            visited_stack.push(*candidate);
        }
    }
}
