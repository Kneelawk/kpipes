use bytemuck::{Pod, Zeroable};
use std::mem::size_of;
use wgpu::{
    BufferAddress, InputStepMode, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
};

/// Vertex data.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    pub fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: size_of::<Vertex>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float3,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    format: VertexFormat::Float3,
                    shader_location: 1,
                },
            ],
        }
    }
}
