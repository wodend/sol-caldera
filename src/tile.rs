use std::fs;
use std::path::PathBuf;

use crate::model;
use sol_grid::{vox, Grid, Voxel, Rotation};

pub struct Tile {
    name: &'static str,
    voxels: Grid<Voxel>,
    symmetry: Symmetry,
}

pub enum Symmetry {
    All,
    None,
}

impl Tile {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn voxels(&self) -> &Grid<Voxel> {
        &self.voxels
    }

    pub fn symmetry(&self) -> &Symmetry {
        &self.symmetry
    }
}


#[derive(Debug)]
enum Transform {
    Rotation(Rotation),
}

fn transform(model: Tile, transforms: Vec<Transform>) {
    let voxels = model.voxels();
    for transform in transforms {
        let path = PathBuf::from("models")
            .join(format!("{}_{:?}", model.name(), transform))
            .with_extension("vox");
        let transformed = match transform {
            Transform::Rotation(r) => voxels.rotated_z(&r),
        };
        let bytes = vox::encode(&transformed).unwrap();
        fs::write(path, &bytes).unwrap();
    }
}

pub fn gen(tile_set_template: TileSetTemplate) {
    for tile in tile_set_template.tiles() {
        let transforms = match tile.symmetry() {
            Symmetry::None => vec![
                Transform::Rotation(Rotation::R0),
                Transform::Rotation(Rotation::R90),
                Transform::Rotation(Rotation::R180),
                Transform::Rotation(Rotation::R270),
            ],
            _ => vec![Transform::Rotation(Rotation::R0)],
        };
        transform(tile, transforms);
    }
}

pub enum TileSetTemplate {
    Road,
}

impl TileSetTemplate {
    fn tiles(&self) -> Vec<Tile> {
        match *self {
            TileSetTemplate::Road => {
                let (width, depth, height) = (3, 3, 3);
                vec![
                    Tile {
                        name: "dirt",
                        voxels: model::gen::dirt(width, depth, height),
                        symmetry: Symmetry::All,
                    },
                    Tile {
                        name: "grass",
                        voxels: model::gen::grass(width, depth, height),
                        symmetry: Symmetry::All,
                    },
                    Tile {
                        name: "road-inner",
                        voxels: model::gen::road_inner(width, depth, height),
                        symmetry: Symmetry::All,
                    },
                    Tile {
                        name: "road-edge",
                        voxels: model::gen::road_edge(width, depth, height),
                        symmetry: Symmetry::None,
                    },
                    Tile {
                        name: "road-corner",
                        voxels: model::gen::road_corner(width, depth, height),
                        symmetry: Symmetry::None,
                    },
                    Tile {
                        name: "sky",
                        voxels: model::gen::sky(width, depth, height),
                        symmetry: Symmetry::All,
                    },
                ]
            }
        }
    }
}