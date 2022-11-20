use std::fmt::{self, Display, Formatter};
use std::collections::HashMap;

use crate::{model, map::Direction};
use sol_grid::{Grid, Rotation, Voxel};

pub struct Tile {
    name: String,
    voxels: Grid<Voxel>,
    tag: Tag,
    orientation: Orientation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Dirt,
    Grass,
    Sky,
    Road,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Invariant,
    Edge(Direction),
    Corner(Direction),
}

impl Orientation {
    fn rotated_z(&self, rotation: &Rotation) -> Self {
        match self {
            Self::Edge(d) => Self::Edge(d.rotated_z(rotation)),
            Self::Corner(d) => Self::Corner(d.rotated_z(rotation)),
            other => *other,
        }
    }
}

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Edge(d) => write!(f, "{}", format!("{:?}", d).to_lowercase()),
            Self::Corner(d) => write!(f, "{}", format!("{:?}", d).to_lowercase()),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

impl Tile {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn voxels(&self) -> &Grid<Voxel> {
        &self.voxels
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn orientation(&self) -> &Orientation {
        &self.orientation
    }

    pub fn rotated_z(&self, rotation: &Rotation) -> Self {
        let voxels = self.voxels().rotated_z(rotation);
        let orientation = self.orientation().rotated_z(rotation);
        Self {
            name: format!("{}-{}", self.name, orientation),
            voxels: voxels,
            tag: *self.tag(),
            orientation: orientation,
        }
    }
}

pub struct TileSet {
    seed_id: u32,
    tiles: Vec<Tile>,
    updates: HashMap<(u32, Direction), Vec<f32>>,
}

impl TileSet {
    pub fn gen(template: Template) -> Self {
        let mut tiles = Vec::new();
        for tile in template.tiles() {
            tiles.extend(
                match tile.orientation {
                    Orientation::Invariant => vec![tile],
                    _ => vec![
                        tile.rotated_z(&Rotation::R0),
                        tile.rotated_z(&Rotation::R90),
                        tile.rotated_z(&Rotation::R180),
                        tile.rotated_z(&Rotation::R270),
                    ],
                }
            );
        }
        let mut updates = HashMap::new();
        let directions = [
            Direction::East,
            Direction::West,
            Direction::North,
            Direction::South,
        ];
        const BAN: f32 = -1.0;
        for (id, source) in tiles.iter().enumerate() {
            for direction in directions {
                let mut update = vec![0.0; tiles.len()];
                for target in &tiles {
                    update.push(
                        match source.tag() {
                            Tag::Dirt => match target.tag() {
                                Tag::Dirt => if direction.is_horizontal() { 0.8 } else { BAN },
                                Tag::Sky => if matches!(direction, Direction::Up) {
                                    1.0
                                } else if direction.is_horizontal() {
                                    0.2
                                } else {
                                    BAN
                                },
                                // Tag::Grass => if matches!(direction, Direction::Up) { 0.1 } else { BAN },
                                _ => BAN,
                            },
                            _ => BAN,
                            // Tag::Grass => match target.tag() {
                            //     Tag::Grass => if direction.is_horizontal() { 0.1 } else { BAN },
                            //     Tag::Sky => if matches!(direction, Direction::Up) { 0.1 } else { BAN },
                            //     Tag::Road => if matches!(direction, Direction::Up) { 0.1 } else { BAN },
                            //     _ => BAN,
                            // },
                            // Tag::Sky => match target.tag() {
                            //     Tag::Sky => 0.1,
                            //     Tag::Road => match (direction, target.orientation()) {
                            //         (d, Orientation::Edge(t)) => if d == *t { 0.1 } else { BAN },
                            //         (
                            //             Direction::East,
                            //             Orientation::Corner(Direction::NorthWest)
                            //             | Orientation::Corner(Direction::SouthWest)
                            //         ) => 0.1,
                            //         (
                            //             Direction::West,
                            //             Orientation::Corner(Direction::NorthEast)
                            //             | Orientation::Corner(Direction::SouthEast)
                            //         ) => 0.1,
                            //         (
                            //             Direction::North,
                            //             Orientation::Corner(Direction::SouthEast)
                            //             | Orientation::Corner(Direction::SouthWest)
                            //         ) => 0.1,
                            //         (
                            //             Direction::South,
                            //             Orientation::Corner(Direction::NorthEast)
                            //             | Orientation::Corner(Direction::NorthWest)
                            //         ) => 0.1,
                            //         _ => BAN,
                            //     },
                            //     _ => BAN,
                            // },
                            // Tag::Road => match target.tag() {
                            //     Tag::Road => {
                            //         match (source.orientation(), direction, target.orientation()) {
                            //             (Orientation::Invariant, _, Orientation::Invariant) => 0.1,
                            //             (Orientation::Invariant, d, Orientation::Edge(t)) => {
                            //                 if d == t.rotated_z(&Rotation::R180) {
                            //                     0.1
                            //                 } else {
                            //                     BAN
                            //                 }
                            //             },
                            //             (Orientation::Edge(s), d, Orientation::Edge(t)) => {
                            //                 if *s == d && *t == s.rotated_z(&Rotation::R180) {
                            //                     0.1
                            //                 } else if s.is_perpendicular(d) && *s == *t {
                            //                     0.1
                            //                 } else {
                            //                     BAN
                            //                 }
                            //             },
                            //             // TODO Finish constraints
                            //             (Orientation::Edge(s), d, Orientation::Corner(t)) => {
                            //                 if *s == d && *t == s.rotated_z(&Rotation::R180) {
                            //                     0.1
                            //                 } else if s.is_perpendicular(d) && *s == *t {
                            //                     0.1
                            //                 } else {
                            //                     BAN
                            //                 }
                            //             },
                            //             _ => BAN,
                            //         }
                            //     },
                            //     _ => BAN,
                            // },
                        }
                    );
                }
                updates.insert((id as u32, direction), update);
            }
        }
        Self {
            seed_id: 0,
            tiles: tiles,
            updates: updates,
        }
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn seed_id(&self) -> u32 {
        self.seed_id
    }

    pub fn update(&self, tile_id: u32, direction: Direction) -> &Vec<f32> {
        &self.updates[&(tile_id, direction)]
    }

    pub fn voxels(&self, tile_id: u32) -> &Grid<Voxel> {
        &self.tiles[tile_id as usize].voxels()
    }
}


#[derive(Debug)]
pub enum Template {
    Road,
}

impl Template {
    fn tiles(&self) -> Vec<Tile> {
        match *self {
            Template::Road => {
                let (width, depth, height) = (3, 3, 3);
                vec![
                    Tile {
                        name: "dirt".to_string(),
                        voxels: model::gen::dirt(width, depth, height),
                        tag: Tag::Dirt,
                        orientation: Orientation::Invariant,
                    },
                    Tile {
                        name: "grass".to_string(),
                        voxels: model::gen::grass(width, depth, height),
                        tag: Tag::Grass,
                        orientation: Orientation::Invariant,
                    },
                    Tile {
                        name: "road-inner".to_string(),
                        voxels: model::gen::road_inner(width, depth, height),
                        tag: Tag::Road,
                        orientation: Orientation::Invariant,
                    },
                    Tile {
                        name: "road-edge".to_string(),
                        voxels: model::gen::road_edge(width, depth, height),
                        tag: Tag::Road,
                        orientation: Orientation::Edge(Direction::West),
                    },
                    Tile {
                        name: "road-corner".to_string(),
                        voxels: model::gen::road_corner(width, depth, height),
                        tag: Tag::Road,
                        orientation: Orientation::Corner(Direction::NorthEast),
                    },
                    Tile {
                        name: "sky".to_string(),
                        voxels: model::gen::sky(width, depth, height),
                        tag: Tag::Sky,
                        orientation: Orientation::Invariant,
                    },
                ]
            }
        }
    }
}
