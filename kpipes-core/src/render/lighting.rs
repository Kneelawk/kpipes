use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Vector3};

pub const NUM_LIGHTS: usize = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub direction: Vector3<f32>,
    pub strength: f32,
    pub _padding1: u32,
    pub _padding2: u32,
    pub _padding3: u32,
}

impl Light {
    pub fn new(direction: Vector3<f32>, strength: f32) -> Light {
        Light {
            direction: direction.normalize(),
            strength,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}

/// Manages the scene's global lights.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Lighting {
    // Everything is in this specific order because of the weird differences between GLSL alignment
    // and C alignment.
    pub lights: [Light; NUM_LIGHTS],
    pub ambient_light: f32,
}

unsafe impl Pod for Lighting {}
unsafe impl Zeroable for Lighting {}

impl Lighting {
    pub fn new(lights: [Light; NUM_LIGHTS], ambient_light: f32) -> Lighting {
        Lighting {
            lights,
            ambient_light,
        }
    }
}
