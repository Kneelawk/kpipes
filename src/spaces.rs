use crate::{SPACE_DEPTH, SPACE_HEIGHT, SPACE_WIDTH};
use cgmath::Vector3;
use std::mem::size_of;

const CELL_BITS: usize = size_of::<u64>() * 8;

#[derive(Debug, Clone)]
pub struct Spaces {
    spaces: Vec<u64>,
}

impl Default for Spaces {
    fn default() -> Self {
        Spaces {
            // round up division
            spaces: vec![0; (SPACE_WIDTH * SPACE_HEIGHT * SPACE_DEPTH - 1) / CELL_BITS + 1],
        }
    }
}

impl Spaces {
    /// Clears all the spaces.
    pub fn clear(&mut self) {
        for space in self.spaces.iter_mut() {
            *space = 0;
        }
    }

    /// Sets a space to occupied.
    pub fn set(&mut self, x: usize, y: usize, z: usize) {
        if x >= SPACE_WIDTH || y >= SPACE_HEIGHT || z >= SPACE_DEPTH {
            panic!("Setting a space out of bounds: ({}, {}, {})", x, y, z);
        }

        self.spaces[(x + y * SPACE_WIDTH + z * SPACE_WIDTH * SPACE_HEIGHT) / CELL_BITS] |=
            1 << ((x + y * SPACE_WIDTH + z * SPACE_WIDTH * SPACE_HEIGHT) % CELL_BITS);
    }

    /// Sets a space to occupied.
    pub fn set_vec(&mut self, loc: Vector3<usize>) {
        self.set(loc.x, loc.y, loc.z);
    }

    /// Gets whether a space is occupied.
    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        if x >= SPACE_WIDTH || y >= SPACE_HEIGHT || z >= SPACE_DEPTH {
            panic!("Setting a space out of bounds: ({}, {}, {})", x, y, z);
        }

        let one = 1 << ((x + y * SPACE_WIDTH + z * SPACE_WIDTH * SPACE_HEIGHT) % CELL_BITS);

        return self.spaces[(x + y * SPACE_WIDTH + z * SPACE_WIDTH * SPACE_HEIGHT) / CELL_BITS] & one
            == one;
    }

    /// Gets whether a space is occupied.
    pub fn get_vec(&self, loc: Vector3<usize>) -> bool {
        self.get(loc.x, loc.y, loc.z)
    }
}
