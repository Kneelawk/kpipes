use crate::render::{buffer::BufferWrapper, vertex::Vertex};
use cgmath::Vector3;
use std::io::BufRead;
use tobj::{load_obj_buf, LoadError};
use wgpu::{BufferUsage, CommandBuffer, Device, RenderPass};

/// Describes a single mesh.
pub struct Mesh {
    vertex_buffer: BufferWrapper<Vertex>,
    index_buffer: BufferWrapper<u32>,
}

impl Mesh {
    /// Loads a mesh from memory containing a wavefront obj format object file.
    pub fn load<B: BufRead>(
        device: &Device,
        reader: &mut B,
    ) -> Result<(Mesh, Vec<CommandBuffer>), MeshLoadError> {
        let (obj, _) = load_obj_buf(reader, true, |_p| Err(LoadError::GenericFailure))?;

        let model = obj
            .into_iter()
            .next()
            .ok_or(MeshLoadError::MissingModelError)?;

        let mut vertices = vec![];

        for i in 0..(model.mesh.positions.len() / 3) {
            vertices.push(Vertex {
                position: Vector3::new(
                    model.mesh.positions[i * 3],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ),
                normal: Vector3::new(
                    model.mesh.normals[i * 3],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2],
                ),
            })
        }

        let (vertex_buffer, vertex_cb) =
            BufferWrapper::from_data(device, &vertices, BufferUsage::VERTEX);
        let (index_buffer, index_cb) =
            BufferWrapper::from_data(device, &model.mesh.indices, BufferUsage::INDEX);

        Ok((
            Mesh {
                vertex_buffer,
                index_buffer,
            },
            vec![vertex_cb, index_cb],
        ))
    }

    pub fn index_len(&self) -> u32 {
        self.index_buffer.size() as u32
    }

    /// Bind this model for a subsequent draw call.
    pub fn bind<'a>(&'a self, render_pass: &mut RenderPass<'a>, vertex_slot: u32) {
        render_pass.set_vertex_buffer(vertex_slot, self.vertex_buffer.buffer(), 0, 0);
        render_pass.set_index_buffer(self.index_buffer.buffer(), 0, 0);
    }
}

/// Error potentially returned when loading a mesh.
#[derive(Debug, Copy, Clone)]
pub enum MeshLoadError {
    ObjLoadError(LoadError),
    MissingModelError,
}

impl From<LoadError> for MeshLoadError {
    fn from(e: LoadError) -> Self {
        MeshLoadError::ObjLoadError(e)
    }
}
