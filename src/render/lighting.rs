use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Vector3};

pub const NUM_LIGHTS: usize = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub direction: Vector3<f32>,
    pub strength: f32,
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
    pub fn new() -> Lighting {
        Lighting {
            lights: [
                Light {
                    direction: Vector3::new(-2.0, 3.0, -4.0).normalize(),
                    strength: 1.0,
                },
                Light {
                    direction: Vector3::new(1.0, -2.0, 3.0).normalize(),
                    strength: 0.8,
                },
            ],
            ambient_light: 0.1,
        }
    }
}
