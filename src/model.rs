pub mod gen {
    use sol_grid::{Grid, Voxel};

    const BROWN: [u8; 4] = [120, 80, 50, 255];
    const GREEN: [u8; 4] = [90, 120, 20, 255];
    const GREY: [u8; 4] = [108, 108, 127, 255];
    const CLEAR: [u8; 4] = [0, 0, 0, 0];

    pub fn dirt(width: u32, depth: u32, height: u32) -> Grid<Voxel> {
        let mut voxels = Grid::new(width, depth, height);
        for (_, _, _, v) in voxels.enumerate_cells_mut() {
            *v = Voxel::from_rgba(&BROWN);
        }
        voxels
    }

    pub fn grass(width: u32, depth: u32, height: u32) -> Grid<Voxel> {
        let mut voxels = dirt(width, depth, height);
        for (_, _, z, v) in voxels.enumerate_cells_mut() {
            if z == height - 1 {
                *v = Voxel::from_rgba(&GREEN);
            }
        }
        voxels
    }

    pub fn sky(width: u32, depth: u32, height: u32) -> Grid<Voxel> {
        let mut voxels = Grid::new(width, depth, height);
        for (_, _, _, v) in voxels.enumerate_cells_mut() {
            *v = Voxel::from_rgba(&CLEAR);
        }
        voxels
    }

    pub fn road_inner(width: u32, depth: u32, height: u32) -> Grid<Voxel> {
        let mut voxels = sky(width, depth, height);
        for (_, _, z, v) in voxels.enumerate_cells_mut() {
            if z == 0 {
                *v = Voxel::from_rgba(&GREY);
            }
        }
        voxels
    }

    pub fn road_edge(width: u32, depth: u32, height: u32) -> Grid<Voxel> {
        let mut voxels = sky(width, depth, height);
        for (x, _, z, v) in voxels.enumerate_cells_mut() {
            if z == 0 && x >= width / 2 {
                *v = Voxel::from_rgba(&GREY);
            }
        }
        voxels
    }

    pub fn road_corner(width: u32, depth: u32, height: u32) -> Grid<Voxel> {
        let mut voxels = sky(width, depth, height);
        for (x, y, z, v) in voxels.enumerate_cells_mut() {
            if z == 0 && x <= width / 2 && y <= depth / 2 {
                *v = Voxel::from_rgba(&GREY);
            }
        }
        voxels
    }
}
