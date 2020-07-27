use crate::{SPACE_DEPTH, SPACE_HEIGHT, SPACE_WIDTH};
use cgmath::{One, Vector3};
use std::ops::{Add, Sub};

/// Describes a direction in 3d space.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, IntoEnumIterator)]
pub enum Direction {
    Up,
    Down,
    East,
    West,
    South,
    North,
}

impl Direction {
    /// Moves a Vector3 one unit along this direction.
    pub fn offset<I>(&self, vec: Vector3<I>) -> Vector3<I>
    where
        I: One<Output = I> + Add<Output = I> + Sub<Output = I>,
    {
        match *self {
            Direction::Up => Vector3::new(vec.x, vec.y + I::one(), vec.z),
            Direction::Down => Vector3::new(vec.x, vec.y - I::one(), vec.z),
            Direction::East => Vector3::new(vec.x + I::one(), vec.y, vec.z),
            Direction::West => Vector3::new(vec.x - I::one(), vec.y, vec.z),
            Direction::South => Vector3::new(vec.x, vec.y, vec.z + I::one()),
            Direction::North => Vector3::new(vec.x, vec.y, vec.z - I::one()),
        }
    }

    pub fn is_offset_legal(&self, vec: Vector3<usize>) -> bool {
        match *self {
            Direction::Up => vec.y < SPACE_HEIGHT - 1,
            Direction::Down => vec.y > 0,
            Direction::East => vec.x < SPACE_WIDTH - 1,
            Direction::West => vec.x > 0,
            Direction::South => vec.z < SPACE_DEPTH - 1,
            Direction::North => vec.z > 0,
        }
    }
}
