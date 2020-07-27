use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
    TextureView,
};
use winit::dpi::PhysicalSize;

pub struct TextureWrapper {
    pub texture: Texture,
    pub view: TextureView,
}

impl TextureWrapper {
    pub fn new_depth(device: &Device, size: PhysicalSize<u32>, label: &str) -> TextureWrapper {
        let extent = Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };

        let desc = TextureDescriptor {
            label: Some(label),
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::COPY_SRC,
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_default_view();

        TextureWrapper { texture, view }
    }
}
