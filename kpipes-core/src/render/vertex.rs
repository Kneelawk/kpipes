use crate::render::VertexData;
use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use std::mem::size_of;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

/// Vertex data.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    const ATTRS: [VertexAttribute; 2] = wgpu::vertex_attr_array![5 => Float32x3, 6 => Float32x3];
}

impl VertexData for Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Vertex::ATTRS,
        }
    }
}
