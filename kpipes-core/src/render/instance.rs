use crate::render::VertexData;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Vector3};
use std::mem::size_of;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

/// Instance data.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    pub color: Vector3<f32>,
    pub model: Matrix4<f32>,
}

unsafe impl Pod for Instance {}
unsafe impl Zeroable for Instance {}

impl Instance {
    const ATTRS: [VertexAttribute; 5] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4];
}

impl VertexData for Instance {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Instance::ATTRS,
        }
    }
}
