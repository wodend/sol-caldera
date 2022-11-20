use std::fs;
use std::collections::HashSet;
use std::os::windows::thread;
use std::path::PathBuf;

use rand::distributions::{Distribution, Uniform, WeightedIndex};
use rand::rngs::ThreadRng;
use rand::thread_rng;


use sol_grid::{vox, Grid, Rotation, Voxel};
use crate::tile::{Template, TileSet};
use crate::math::{add, normalize, entropy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    East,
    West,
    North,
    South,
    Up,
    Down,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub fn rotated_z(&self, rotation: &Rotation) -> Self {
        match self {
            Self::East => match rotation {
                Rotation::R0 => Self::East,
                Rotation::R90 => Self::North,
                Rotation::R180 => Self::West,
                Rotation::R270 => Self::South,
            },
            Self::West => match rotation {
                Rotation::R0 => Self::West,
                Rotation::R90 => Self::South,
                Rotation::R180 => Self::East,
                Rotation::R270 => Self::North,
            },
            Self::North => match rotation {
                Rotation::R0 => Self::North,
                Rotation::R90 => Self::West,
                Rotation::R180 => Self::South,
                Rotation::R270 => Self::East,
            },
            Self::South => match rotation {
                Rotation::R0 => Self::South,
                Rotation::R90 => Self::East,
                Rotation::R180 => Self::North,
                Rotation::R270 => Self::West,
            },
            Self::NorthEast => match rotation {
                Rotation::R0 => Self::NorthEast,
                Rotation::R90 => Self::NorthWest,
                Rotation::R180 => Self::SouthWest,
                Rotation::R270 => Self::SouthEast,
            },
            Self::NorthWest => match rotation {
                Rotation::R0 => Self::NorthWest,
                Rotation::R90 => Self::SouthWest,
                Rotation::R180 => Self::SouthEast,
                Rotation::R270 => Self::NorthEast,
            },
            Self::SouthEast => match rotation {
                Rotation::R0 => Self::SouthEast,
                Rotation::R90 => Self::NorthEast,
                Rotation::R180 => Self::NorthWest,
                Rotation::R270 => Self::SouthWest,
            },
            Self::SouthWest => match rotation {
                Rotation::R0 => Self::SouthWest,
                Rotation::R90 => Self::SouthEast,
                Rotation::R180 => Self::NorthEast,
                Rotation::R270 => Self::NorthWest,
            },
            other => *other,
        }
    }

    pub fn is_perpendicular(&self, other: Direction) -> bool {
        match self {
            Self::East => match other {
                Self::North | Self::South => true,
                _ => false,
            },
            Self::West => match other {
                Self::North | Self::South => true,
                _ => false,
            },
            Self::North => match other {
                Self::East | Self::West => true,
                _ => false,
            },
            Self::South => match other {
                Self::East | Self::West => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        match self {
            Self::East | Self::West | Self::North | Self::South => true,
            _ => false,
        }
    }

    pub fn is_vertical(&self) -> bool {
        match self {
            Self::Up | Self::Down => true,
            _ => false,
        }
    }
}

struct Edge {
    direction: Direction,
    wave: usize,
}

struct Node {
    pub current_cell_id: usize,
    pub current_state_name: &'static str,
    pub current_distance: usize,
}

impl Node {
    pub fn new(
        current_cell_id: usize,
        current_state_name: &'static str,
        current_distance: usize,
    ) -> Node {
        Node {
            current_cell_id: current_cell_id,
            current_state_name: current_state_name,
            current_distance: current_distance,
        }
    }
}

pub struct Map {
    width: u32,
    depth: u32,
    height: u32,
    tileset: TileSet,
    rng: ThreadRng,
    graph: Vec<Vec<Edge>>,
    weights: Vec<Vec<f32>>,
    observations: Vec<Option<u32>>,
}

impl Map {
    pub fn new(width: u32, depth: u32, height: u32, tileset: TileSet) -> Map {
        let rng = thread_rng();
        let len = width as usize * depth as usize * height as usize;
        let graph = Vec::with_capacity(len);
        let weights = Vec::with_capacity(len);
        let observations = Vec::with_capacity(len);
        for x in 0..height {
            for y in 0..depth {
                for z in 0..width {
                    graph.push(edges(width, depth, height, x, y, z));
                    weights.push(vec![0.0; tileset.len()]);
                    if x == width / 2 && y == depth / 2 && z == 0 {
                        observations.push(Some(tileset.seed_id()));
                    } else {
                        observations.push(None);
                    }
                }
            }
        }
        Map {
            width: width,
            depth: depth,
            height: height,
            tileset: tileset,
            rng: rng,
            graph: graph,
            weights: weights,
            observations: observations,
        }
    }

    pub fn gen(width: u32, depth: u32, height: u32, tileset: TileSet) -> Map {
        let map = Map::new(width, depth, height, tileset);
        map.wave_function_collapse();
        map
    }

    pub fn wave_function_collapse(&mut self) {
        let directions = [
            Direction::East,
            Direction::West,
            Direction::North,
            Direction::South,
            Direction::Up,
            Direction::Down,
        ];
        let tiles = Vec::from_iter(self.tiles[wave].clone());
        let observed_tile = tiles.choose(&mut self.rng).unwrap();
        self.tiles[wave] = HashSet::from([*observed_tile]);
        while let Some(cell_id) = self.min_entropy_cell_id() {
            self.observe(cell_id)?;
            self.propagate(cell_id);
        }
        Ok(())
    }

    fn observe(&mut self, cell_id: usize) -> Result<(), GenerationError> {
        let distribution = match WeightedIndex::new(&self.cells.weights[cell_id]) {
            Ok(distribution) => distribution,
            Err(_) => return Err(GenerationError::Contradiction),
        };
        let state_id = distribution.sample(&mut self.rng);
        self.cells.observations[cell_id] = Some(self.state_names[state_id]);
        for (id, probability) in self.cells.weights[cell_id].iter_mut().enumerate() {
            if id == state_id {
                *probability = 1.0;
            } else {
                *probability = 0.0;
            }
        }
        self.cells.entropies[cell_id] = 0.0;
        Ok(())
    }

    fn propagate(&mut self, cell_id: usize) {
        let mut stack = match self.cells.observations[cell_id] {
            Some(state_name) => vec![Node::new(cell_id, state_name, 0)],
            None => Vec::new(),
        };
        let mut visited = HashSet::new();
        while let Some(Node {
            current_cell_id,
            current_state_name,
            current_distance,
        }) = stack.pop()
        {
            visited.insert(current_cell_id);
            for Edge { cell_id, direction } in self.cells.edges[current_cell_id].iter() {
                if !visited.contains(cell_id) {
                    let distance = current_distance + 1;
                    // Update cell if not collapsed
                    if self.cells.observations[*cell_id].is_none() {
                        let signal = Signal::new(current_state_name, *direction, distance);
                        let weights = self
                            .states
                            .iter()
                            .map(|state| (state.update_probability)(signal))
                            .collect();
                        add(&mut self.cells.weights[*cell_id], &weights);
                        normalize(&mut self.cells.weights[*cell_id]);
                        self.cells.entropies[*cell_id] =
                            entropy(&self.cells.weights[*cell_id]);
                    }
                    // Continue the propagation up to max_distance
                    if distance < self.max_distance {
                        stack.push(Node::new(*cell_id, current_state_name, distance));
                    }
                }
            }
        }
    }

    pub fn voxels(&self) -> Grid<Voxel> {
        let mut map = Grid::new(map_width, map_depth, map_height);
        for (x, y, z, v) in map.enumerate_cells_mut() {
        }
        let model_size = 3;
        let voxels_width = map.width() * model_size;
        let voxels_depth = map.depth() * model_size;
        let voxels_height = map.height() * model_size;
        let mut voxels = Grid::new(voxels_width, voxels_depth, voxels_height);
        for (mx, my, mz, m) in map.enumerate_cells() {
            let offset_x = mx * model_size;
            let offset_y = my * model_size;
            let offset_z = mz * model_size;
            for (vx, vy, vz, v) in models[*m as usize].enumerate_cells() {
                let x = vx + offset_x;
                let y = vy + offset_y;
                let z = vz + offset_z;
                *voxels.get_mut(x, y, z) = *v;
            }
        }
        voxels
    }
}
