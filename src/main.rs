mod model;
mod tile;
mod map;

use std::fs;
use std::path::PathBuf;
use sol_grid::vox;
use tile::{Template, TileSet};

fn main() {
    let template = Template::Road;
    let tileset = TileSet::gen(template);
    let map = Map::gen(10, 10, 10, tileset);
    let voxels = map.voxels();
    let bytes = vox::encode(voxels).unwrap();
    let path = PathBuf::from("models")
        .join(format!("{:?}", template))
        .with_extension("vox");
    fs::write(path, &bytes).unwrap();
}
