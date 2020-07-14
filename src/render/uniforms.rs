use crate::render::camera::Camera;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};

/// Holds data passed to the shaders in the form of uniforms.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniforms {
    pub vp_matrix: Matrix4<f32>,
}

unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}

impl Uniforms {
    pub fn new() -> Uniforms {
        Uniforms {
            vp_matrix: Matrix4::identity(),
        }
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.vp_matrix = camera.build_vp_matrix();
    }
}
