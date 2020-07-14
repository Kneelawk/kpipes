use winit::{dpi::PhysicalSize, window::Window};

/// Used to manage the details of how render operations are performed.
pub struct RenderEngine {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    sc: wgpu::SwapChain,
    size: PhysicalSize<u32>,
}

impl RenderEngine {
    /// Create a new RenderEngine for the given window.
    ///
    /// Will return a RenderEngineCreationError if an error occurs while
    /// creating the engine.
    pub async fn new(window: &Window) -> Result<RenderEngine, RenderEngineCreationError> {
        let size = window.inner_size();

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .ok_or(RenderEngineCreationError::MissingAdapterError)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let sc = device.create_swap_chain(&surface, &sc_desc);

        Ok(RenderEngine {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            sc,
            size,
        })
    }

    /// Resizes the swap chain for this RenderEngine.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.sc = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    /// Performs a render.
    pub fn render(&mut self) -> Result<(), RenderEngineRenderError> {
        let frame = self.sc.get_next_texture()?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(&[encoder.finish()]);

        Ok(())
    }
}

/// Error potentially returned when creating a RenderEngine.
#[derive(Debug, Clone)]
pub enum RenderEngineCreationError {
    MissingAdapterError,
}

/// Error potentially returned when performing a render operation.
#[derive(Debug, Clone)]
pub enum RenderEngineRenderError {
    TimeOut(wgpu::TimeOut),
}

impl From<wgpu::TimeOut> for RenderEngineRenderError {
    fn from(e: wgpu::TimeOut) -> Self {
        RenderEngineRenderError::TimeOut(e)
    }
}
