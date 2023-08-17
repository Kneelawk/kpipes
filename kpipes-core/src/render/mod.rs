pub mod buffer;
pub mod camera;
pub mod instance;
pub mod instance_manager;
pub mod lighting;
pub mod mesh;
pub mod texture;
pub mod uniforms;
pub mod util;
pub mod vertex;

use crate::{
    messages::FrameSize,
    render::{
        buffer::{BufferRemoveError, BufferWrapper, BufferWriteError},
        camera::Camera,
        instance::Instance,
        instance_manager::{InstanceManager, InstanceManagerCreationError},
        lighting::Lighting,
        texture::TextureWrapper,
        uniforms::Uniforms,
        vertex::Vertex,
    },
};
use std::{borrow::Cow, io, io::BufRead};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferAddress,
    BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CommandBuffer,
    CommandEncoderDescriptor, CompareFunction, DepthStencilState, Device, Face, FragmentState,
    FrontFace, LoadOp, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, TextureFormat, TextureView, VertexBufferLayout, VertexState,
};

const SHADER_VERT_SRC: &str = include_str!("shader.vert.wgsl");
const SHADER_FRAG_SRC: &str = include_str!("shader.frag.wgsl");

/// Used to manage the details of how render operations are performed.
pub struct RenderEngine {
    instance_groups: Vec<InstanceManager>,
    uniforms: Uniforms,
    uniform_buffer: BufferWrapper<Uniforms>,
    // We need to make sure this buffer isn't dropped before this struct is.
    #[allow(dead_code)]
    lighting_buffer: BufferWrapper<Lighting>,
    uniform_bind_group: BindGroup,
    depth_texture: TextureWrapper,
    render_pipeline: RenderPipeline,

    /// This render engine's camera in 3d space.
    pub camera: Camera,
}

impl RenderEngine {
    /// Create a new RenderEngine for the given window.
    ///
    /// Will return a RenderEngineCreationError if an error occurs while
    /// creating the engine.
    pub fn new<B: BufRead>(
        device: &Device,
        queue: &Queue,
        window_size: FrameSize,
        color_format: TextureFormat,
        lighting: Lighting,
        instances: &mut [B],
        instance_capacity: BufferAddress,
    ) -> Result<RenderEngine, RenderEngineCreationError> {
        let mut queue_submissions = vec![];

        // setup instance managers
        let mut instance_groups = vec![];
        for instance_data in instances {
            let (instance_manager, mut cb) =
                InstanceManager::from_obj(device, instance_data, instance_capacity)?;
            instance_groups.push(instance_manager);
            queue_submissions.append(&mut cb);
        }

        // setup camera
        let camera = Camera {
            eye: (0.0, 5.0, 5.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: (0.0, 1.0, 0.0).into(),
            aspect: window_size.width as f32 / window_size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        // setup uniform holder
        let mut uniforms = Uniforms::new();
        uniforms.update_camera(&camera);

        // create uniform buffer
        let (uniform_buffer, uniform_cb) =
            BufferWrapper::from_data(device, &[uniforms], BufferUsages::UNIFORM);
        queue_submissions.push(uniform_cb);

        // setup lighting values
        let (lighting_buffer, lighting_cb) =
            BufferWrapper::from_data(device, &[lighting], BufferUsages::UNIFORM);
        queue_submissions.push(lighting_cb);

        // setup uniform bind group
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(
                        uniform_buffer.buffer().as_entire_buffer_binding(),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(
                        lighting_buffer.buffer().as_entire_buffer_binding(),
                    ),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // setup shaders
        let vs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("vertex_module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(SHADER_VERT_SRC)),
        });
        let fs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("fragment_module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(SHADER_FRAG_SRC)),
        });

        // setup depth texture
        let depth_texture = TextureWrapper::new_depth(device, window_size, "depth_texture");

        // setup render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            multisample: Default::default(),
            vertex: VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[Instance::desc(), Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: color_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multiview: None,
        });

        // submit initial commands
        queue.submit(queue_submissions);

        // return the result
        Ok(RenderEngine {
            instance_groups,
            camera,
            uniforms,
            uniform_buffer,
            lighting_buffer,
            uniform_bind_group,
            depth_texture,
            render_pipeline,
        })
    }

    /// Resizes the swap chain for this RenderEngine.
    pub fn resize(&mut self, device: &Device, window_size: FrameSize) {
        self.camera.aspect = window_size.width as f32 / window_size.height as f32;
        self.depth_texture = TextureWrapper::new_depth(device, window_size, "depth_texture");
    }

    /// Updates the data on the gpu to match the changes to this RenderEngine's
    /// camera.
    pub async fn update_camera(
        &mut self,
        device: &Device,
    ) -> Result<CommandBuffer, BufferWriteError> {
        self.uniforms.update_camera(&self.camera);

        Ok(self
            .uniform_buffer
            .replace_all(device, &[self.uniforms])
            .await?)
    }

    /// Adds instances to this render engine.
    pub async fn add_instances(
        &mut self,
        device: &Device,
        group_index: usize,
        instances: &[Instance],
    ) -> Result<CommandBuffer, BufferWriteError> {
        Ok(self.instance_groups[group_index]
            .add_instances(device, instances)
            .await?)
    }

    /// Removes a number of this render engine's last instances.
    pub fn remove_instances(
        &mut self,
        group_index: usize,
        instances: BufferAddress,
    ) -> Result<(), BufferRemoveError> {
        self.instance_groups[group_index].remove_instances(instances)
    }

    /// Removes all instance from this render engine.
    pub fn clear_instances(&mut self, group_index: usize) {
        self.instance_groups[group_index].clear_instances();
    }

    /// Performs a render.
    pub fn render(&mut self, device: &Device, view: &TextureView) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render_pass_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            for group in self.instance_groups.iter() {
                group.draw(&mut render_pass);
            }
        }

        encoder.finish()
    }
}

/// Trait implemented by anything that can be put into a vertex buffer.
pub trait VertexData {
    fn desc<'a>() -> VertexBufferLayout<'a>;
}

/// Error potentially returned when creating a RenderEngine.
#[derive(Debug)]
pub enum RenderEngineCreationError {
    InstanceManagerCreationError(InstanceManagerCreationError),
    IOError(io::Error),
}

impl From<InstanceManagerCreationError> for RenderEngineCreationError {
    fn from(e: InstanceManagerCreationError) -> Self {
        RenderEngineCreationError::InstanceManagerCreationError(e)
    }
}

impl From<io::Error> for RenderEngineCreationError {
    fn from(e: io::Error) -> Self {
        RenderEngineCreationError::IOError(e)
    }
}
