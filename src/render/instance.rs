use crate::render::VertexData;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Vector3};
use std::mem::size_of;
use wgpu::{
    BufferAddress, InputStepMode, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
};

const FLOAT_SIZE: BufferAddress = size_of::<f32>() as BufferAddress;

/// Instance data.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    pub color: Vector3<f32>,
    pub model: Matrix4<f32>,
}

unsafe impl Pod for Instance {}
unsafe impl Zeroable for Instance {}

impl VertexData for Instance {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: size_of::<Instance>() as BufferAddress,
            step_mode: InputStepMode::Instance,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float3,
                    shader_location: 1,
                },
                VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 3,
                    format: VertexFormat::Float4,
                    shader_location: 2,
                },
                VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 3 + FLOAT_SIZE * 4,
                    format: VertexFormat::Float4,
                    shader_location: 3,
                },
                VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 3 + FLOAT_SIZE * 4 * 2,
                    format: VertexFormat::Float4,
                    shader_location: 4,
                },
                VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 3 + FLOAT_SIZE * 4 * 3,
                    format: VertexFormat::Float4,
                    shader_location: 5,
                },
            ],
        }
    }
}
