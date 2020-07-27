use wgpu::{Device, Queue};

/// A render context houses references to the required components for rendering.
#[derive(Copy, Clone)]
pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
}
