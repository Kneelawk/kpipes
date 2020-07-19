pub mod buffer;
pub mod camera;
pub mod instance;
pub mod texture;
pub mod uniforms;
pub mod vertex;

use crate::render::{
    buffer::{BufferWrapper, BufferWriteError},
    camera::Camera,
    instance::Instance,
    texture::TextureWrapper,
    uniforms::Uniforms,
    vertex::Vertex,
};
use std::{io, io::Cursor, mem::size_of};
use wgpu::{
    read_spirv, Adapter, BackendBit, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Binding, BindingResource, BindingType, BlendDescriptor, BufferAddress,
    BufferUsage, Color, ColorStateDescriptor, ColorWrite, CommandEncoderDescriptor,
    CompareFunction, CullMode, DepthStencilStateDescriptor, Device, DeviceDescriptor, Extensions,
    FrontFace, IndexFormat, LoadOp, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveTopology, ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderStage, StencilStateFaceDescriptor, StoreOp, Surface, SwapChain, SwapChainDescriptor,
    TextureFormat, TextureUsage, TimeOut, VertexBufferDescriptor, VertexStateDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

const SHADER_VERT: &[u8] = include_bytes!("shader.vert.spv");
const SHADER_FRAG: &[u8] = include_bytes!("shader.frag.spv");

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const INDICES: &[u16] = &[
    0, 3, 1,
    0, 2, 3,
    5, 6, 4,
    5, 7, 6,
    1, 4, 0,
    1, 5, 4,
    7, 2, 6,
    7, 3, 2,
    4, 2, 0,
    4, 6, 2,
    1, 7, 5,
    1, 3, 7,
];

/// Used to manage the details of how render operations are performed.
pub struct RenderEngine {
    surface: Surface,
    device: Device,
    queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    vertex_buffer: BufferWrapper<Vertex>,
    index_buffer: BufferWrapper<u16>,
    instance_buffer: BufferWrapper<Instance>,
    uniforms: Uniforms,
    uniform_buffer: BufferWrapper<Uniforms>,
    uniform_bind_group: BindGroup,
    depth_texture: TextureWrapper,
    render_pipeline: RenderPipeline,
    window_size: PhysicalSize<u32>,

    /// This render engine's camera in 3d space.
    pub camera: Camera,
}

impl RenderEngine {
    /// Create a new RenderEngine for the given window.
    ///
    /// Will return a RenderEngineCreationError if an error occurs while
    /// creating the engine.
    pub async fn new(window: &Window) -> Result<RenderEngine, RenderEngineCreationError> {
        let window_size = window.inner_size();

        // setup surface
        let surface = Surface::create(window);

        // setup adapter
        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            BackendBit::PRIMARY,
        )
        .await
        .ok_or(RenderEngineCreationError::MissingAdapterError)?;

        // setup device
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        // setup swap chain
        let sc_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // setup vertex/index/instance data buffers
        let vertex_buffer = BufferWrapper::from_data(&device, VERTICES, BufferUsage::VERTEX);

        let index_buffer = BufferWrapper::from_data(&device, INDICES, BufferUsage::INDEX);

        let instance_buffer = BufferWrapper::new(&device, 3, BufferUsage::VERTEX);

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
        let uniform_buffer = BufferWrapper::from_data(&device, &[uniforms], BufferUsage::UNIFORM);

        // setup uniform bind group
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: uniform_buffer.buffer(),
                    range: 0..size_of::<Uniforms>() as BufferAddress,
                },
            }],
            label: Some("uniform_bind_group"),
        });

        // setup shaders
        let vs_data = read_spirv(Cursor::new(SHADER_VERT))?;
        let fs_data = read_spirv(Cursor::new(SHADER_FRAG))?;

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        // setup depth texture
        let depth_texture = TextureWrapper::new_depth(&device, window_size, "depth_texture");

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
                format: sc_desc.format,
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
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc(), Instance::desc()],
            },
            sample_count: 1,
            sample_mask: 0,
            alpha_to_coverage_enabled: false,
        });

        // return the result
        Ok(RenderEngine {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            camera,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            render_pipeline,
            window_size,
        })
    }

    /// Resizes the swap chain for this RenderEngine.
    pub fn resize(&mut self, window_size: PhysicalSize<u32>) {
        self.window_size = window_size;
        self.sc_desc.width = window_size.width;
        self.sc_desc.height = window_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.camera.aspect = window_size.width as f32 / window_size.height as f32;
        self.depth_texture = TextureWrapper::new_depth(&self.device, window_size, "depth_texture");
    }

    /// Updates the data on the gpu to match the changes to this RenderEngine's
    /// camera.
    pub async fn update_camera(&mut self) -> Result<(), BufferWriteError> {
        self.uniforms.update_camera(&self.camera);

        self.queue.submit(&[self
            .uniform_buffer
            .replace_all(&self.device, &[self.uniforms])
            .await?]);

        Ok(())
    }

    /// Adds an instance to this render engine.
    pub async fn add_instance(&mut self, instance: Instance) -> Result<(), BufferWriteError> {
        self.queue.submit(&[self
            .instance_buffer
            .append(&self.device, &[instance])
            .await?]);

        Ok(())
    }

    /// Removes all instance from this render engine.
    pub fn clear_instances(&mut self) {
        self.instance_buffer.clear();
    }

    /// Performs a render.
    pub fn render(&mut self) -> Result<(), RenderError> {
        let frame = self.swap_chain.get_next_texture()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render_pass_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
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
            render_pass.set_vertex_buffer(0, &self.vertex_buffer.buffer(), 0, 0);
            render_pass.set_vertex_buffer(1, self.instance_buffer.buffer(), 0, 0);
            render_pass.set_index_buffer(&self.index_buffer.buffer(), 0, 0);
            render_pass.draw_indexed(
                0..(self.index_buffer.size() as u32),
                0,
                0..(self.instance_buffer.size() as u32),
            );
        }

        self.queue.submit(&[encoder.finish()]);

        Ok(())
    }
}

/// Trait implemented by anything that can be put into a vertex buffer.
pub trait VertexData {
    fn desc<'a>() -> VertexBufferDescriptor<'a>;
}

/// Error potentially returned when creating a RenderEngine.
#[derive(Debug)]
pub enum RenderEngineCreationError {
    MissingAdapterError,
    IOError(io::Error),
}

impl From<io::Error> for RenderEngineCreationError {
    fn from(e: io::Error) -> Self {
        RenderEngineCreationError::IOError(e)
    }
}

/// Error potentially returned when performing a render operation.
#[derive(Debug, Clone)]
pub enum RenderError {
    TimeOut(TimeOut),
}

impl From<TimeOut> for RenderError {
    fn from(e: TimeOut) -> Self {
        RenderError::TimeOut(e)
    }
}
