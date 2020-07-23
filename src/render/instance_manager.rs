use crate::render::{
    buffer::{BufferWrapper, BufferWriteError},
    instance::Instance,
    mesh::{Mesh, MeshLoadError},
};
use std::io::BufRead;
use wgpu::{BufferAddress, BufferUsage, CommandBuffer, Device, RenderPass};

/// Manages a set of instances of a mesh.
pub struct InstanceManager {
    mesh: Mesh,
    instance_buffer: BufferWrapper<Instance>,
}

impl InstanceManager {
    /// Creates a new instance manager from the given wavefront obj and with the
    /// given instance capacity.
    pub fn from_obj<B: BufRead>(
        device: &Device,
        reader: &mut B,
        instance_capacity: BufferAddress,
    ) -> Result<(InstanceManager, Vec<CommandBuffer>), InstanceManagerCreationError> {
        let (mesh, mesh_cb) = Mesh::load(device, reader)?;

        let instance_buffer = BufferWrapper::new(device, instance_capacity, BufferUsage::VERTEX);

        Ok((
            InstanceManager {
                mesh,
                instance_buffer,
            },
            mesh_cb,
        ))
    }

    /// Appends an instance to this InstanceManager.
    pub async fn add_instance(
        &mut self,
        device: &Device,
        instance: Instance,
    ) -> Result<Vec<CommandBuffer>, BufferWriteError> {
        Ok(vec![
            self.instance_buffer.append(device, &[instance]).await?,
        ])
    }

    /// Removes all instances from this InstanceManager.
    pub fn clear_instances(&mut self) {
        self.instance_buffer.clear();
    }

    /// Draws all the instances managed by this InstanceManager.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.instance_buffer.buffer(), 0, 0);
        self.mesh.bind(render_pass, 1);
        render_pass.draw_indexed(
            0..self.mesh.index_len(),
            0,
            0..(self.instance_buffer.size() as u32),
        );
    }
}

/// Error potentially returned when creating an InstanceManager.
#[derive(Debug, Copy, Clone)]
pub enum InstanceManagerCreationError {
    MeshLoadError(MeshLoadError),
}

impl From<MeshLoadError> for InstanceManagerCreationError {
    fn from(e: MeshLoadError) -> Self {
        InstanceManagerCreationError::MeshLoadError(e)
    }
}
