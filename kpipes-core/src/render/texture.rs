use crate::messages::FrameSize;
use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};

pub struct TextureWrapper {
    pub texture: Texture,
    pub view: TextureView,
}

impl TextureWrapper {
    pub fn new_depth(device: &Device, size: FrameSize, label: &str) -> TextureWrapper {
        let extent = Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let desc = TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[TextureFormat::Depth32Float],
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&TextureViewDescriptor::default());

        TextureWrapper { texture, view }
    }
}
