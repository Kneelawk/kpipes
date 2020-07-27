pub mod buffer;
pub mod camera;
pub mod instance;
pub mod instance_manager;
pub mod lighting;
pub mod mesh;
pub mod texture;
pub mod uniforms;
pub mod vertex;

use crate::{
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
    render_context::RenderContext,
};
use std::{
    io,
    io::{BufRead, Cursor},
    mem::size_of,
};
use wgpu::{
    read_spirv, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    Binding, BindingResource, BindingType, BlendDescriptor, BufferAddress, BufferUsage, Color,
    ColorStateDescriptor, ColorWrite, CommandEncoderDescriptor, CompareFunction, CullMode,
    DepthStencilStateDescriptor, FrontFace, IndexFormat, LoadOp, PipelineLayoutDescriptor,
    PrimitiveTopology, ProgrammableStageDescriptor, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStage,
    StencilStateFaceDescriptor, StoreOp, TextureFormat, TextureView, VertexBufferDescriptor,
    VertexStateDescriptor,
};
use crate::messages::FrameSize;

const SHADER_VERT: &[u8] = include_bytes!("shader.vert.spv");
const SHADER_FRAG: &[u8] = include_bytes!("shader.frag.spv");

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
        render_context: RenderContext<'_>,
        window_size: FrameSize,
        color_format: TextureFormat,
        lighting: Lighting,
        instances: &mut [B],
        instance_capacity: BufferAddress,
    ) -> Result<RenderEngine, RenderEngineCreationError> {
        let device = render_context.device;
        let queue = render_context.queue;

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
            BufferWrapper::from_data(device, &[uniforms], BufferUsage::UNIFORM);
        queue_submissions.push(uniform_cb);

        // setup lighting values
        let (lighting_buffer, lighting_cb) =
            BufferWrapper::from_data(device, &[lighting], BufferUsage::UNIFORM);
        queue_submissions.push(lighting_cb);

        // setup uniform bind group
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::VERTEX,
                        ty: BindingType::UniformBuffer { dynamic: false },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::UniformBuffer { dynamic: false },
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: uniform_buffer.buffer(),
                        range: 0..size_of::<Uniforms>() as BufferAddress,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: lighting_buffer.buffer(),
                        range: 0..size_of::<Lighting>() as BufferAddress,
                    },
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // setup shaders
        let vs_data = read_spirv(Cursor::new(SHADER_VERT))?;
        let fs_data = read_spirv(Cursor::new(SHADER_FRAG))?;

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        // setup depth texture
        let depth_texture = TextureWrapper::new_depth(device, window_size, "depth_texture");

        // setup render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: color_format,
                alpha_blend: BlendDescriptor::REPLACE,
                color_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil_front: StencilStateFaceDescriptor::IGNORE,
                stencil_back: StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[Instance::desc(), Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: 0,
            alpha_to_coverage_enabled: false,
        });

        // submit initial commands
        queue.submit(&queue_submissions);

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
    pub fn resize(&mut self, render_context: RenderContext<'_>, window_size: FrameSize) {
        self.camera.aspect = window_size.width as f32 / window_size.height as f32;
        self.depth_texture =
            TextureWrapper::new_depth(render_context.device, window_size, "depth_texture");
    }

    /// Updates the data on the gpu to match the changes to this RenderEngine's
    /// camera.
    pub async fn update_camera(
        &mut self,
        render_context: RenderContext<'_>,
    ) -> Result<(), BufferWriteError> {
        self.uniforms.update_camera(&self.camera);

        render_context.queue.submit(&[self
            .uniform_buffer
            .replace_all(render_context.device, &[self.uniforms])
            .await?]);

        Ok(())
    }

    /// Adds instances to this render engine.
    pub async fn add_instances(
        &mut self,
        render_context: RenderContext<'_>,
        group_index: usize,
        instances: &[Instance],
    ) -> Result<(), BufferWriteError> {
        render_context.queue.submit(
            &self.instance_groups[group_index]
                .add_instances(render_context.device, instances)
                .await?,
        );

        Ok(())
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
    pub fn render(&mut self, render_context: RenderContext<'_>, view: &TextureView) {
        let mut encoder = render_context
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render_pass_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.02,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_stencil: 0,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            for group in self.instance_groups.iter() {
                group.draw(&mut render_pass);
            }
        }

        render_context.queue.submit(&[encoder.finish()]);
    }
}

/// Trait implemented by anything that can be put into a vertex buffer.
pub trait VertexData {
    fn desc<'a>() -> VertexBufferDescriptor<'a>;
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
