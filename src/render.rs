use std::{io, io::Cursor};
use wgpu::{
    read_spirv, Adapter, BackendBit, BlendDescriptor, Color, ColorStateDescriptor, ColorWrite,
    CommandEncoderDescriptor, CullMode, Device, DeviceDescriptor, Extensions, FrontFace,
    IndexFormat, LoadOp, PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveTopology,
    ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, StoreOp, Surface, SwapChain,
    SwapChainDescriptor, TextureFormat, TextureUsage, TimeOut, VertexStateDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

const SHADER_VERT: &[u8] = include_bytes!("shader.vert.spv");
const SHADER_FRAG: &[u8] = include_bytes!("shader.frag.spv");

/// Used to manage the details of how render operations are performed.
pub struct RenderEngine {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    render_pipeline: RenderPipeline,
    size: PhysicalSize<u32>,
}

impl RenderEngine {
    /// Create a new RenderEngine for the given window.
    ///
    /// Will return a RenderEngineCreationError if an error occurs while
    /// creating the engine.
    pub async fn new(window: &Window) -> Result<RenderEngine, RenderEngineCreationError> {
        let size = window.inner_size();

        let surface = Surface::create(window);

        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            BackendBit::PRIMARY,
        )
        .await
        .ok_or(RenderEngineCreationError::MissingAdapterError)?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_data = read_spirv(Cursor::new(SHADER_VERT))?;
        let fs_data = read_spirv(Cursor::new(SHADER_FRAG))?;

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[],
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
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: 0,
            alpha_to_coverage_enabled: false,
        });

        Ok(RenderEngine {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            size,
        })
    }

    /// Resizes the swap chain for this RenderEngine.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    /// Performs a render.
    pub fn render(&mut self) -> Result<(), RenderEngineRenderError> {
        let frame = self.swap_chain.get_next_texture()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
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
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);

        Ok(())
    }
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
pub enum RenderEngineRenderError {
    TimeOut(TimeOut),
}

impl From<TimeOut> for RenderEngineRenderError {
    fn from(e: TimeOut) -> Self {
        RenderEngineRenderError::TimeOut(e)
    }
}
