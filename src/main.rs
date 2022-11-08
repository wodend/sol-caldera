mod model;
mod tile;

use sol_grid::{vox, Grid, Voxel};
use std::fs;
use std::path::PathBuf;
use tile::TileSetTemplate;

struct Tile {
    p_initial: fn(location: Location) -> f32,
    p_update: fn(tile: Tile, direction: Direction, distance: u32) -> f32,
    tags: Vec<Tag>,
    model_path: PathBuf,
}

enum Location {
    Floor,
    Wall,
}

enum Direction {
    East,
    West,
    South,
    North,
    Down,
    Up,
}

enum Tag {
    Ground,
    Sky,
    Road,
    Rotated0,
    Rotated90,
    Rotated180,
    Rotated270,
    FlippedX,
    FlippedY,
    Asymmetrical,
}

struct Edge {
    direction: Direction,
    wave: usize,
}

pub struct Waves {
    config: Vec<Tile>,
    graph: Vec<Vec<Edge>>,
    weights: Vec<Vec<f32>>,
    entropies: Vec<f32>,
    tiles: Vec<Option<Tile>>,
}

fn gen_test_map(models: &Vec<Grid<Voxel>>) -> Grid<u32> {
    let map_width = 8;
    let map_depth = 8;
    let map_height = 8;
    let mut map = Grid::new(map_width, map_depth, map_height);

    let grass = 0;
    let sky = 1;
    for x in 0..map_width {
        for y in 0..map_depth {
            for z in 0..map_height {
                if z == 0 {
                    *map.get_mut(x, y, z) = grass;
                } else {
                    *map.get_mut(x, y, z) = sky;
                }
            }
        }
    }
    map
}

fn voxels(map: Grid<u32>, models: &Vec<Grid<Voxel>>) -> Grid<Voxel> {
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

fn main() {
    tile::gen(TileSetTemplate::Road);
    // let map = gen_test_map(&models);
    // let voxels = voxels(map, &models);
    // let bytes = vox::encode(voxels).unwrap();
    // fs::write("test_map_voxels.vox", &bytes).unwrap();
}
