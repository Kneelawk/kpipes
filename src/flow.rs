use std::time::{Duration, SystemTime};
use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

/// Used to manage an application's control flow as well as integration with the
/// window manager.
pub struct Flow<Model: 'static> {
    model_init: Box<dyn Fn(&Window) -> Model>,
    event_callback: Option<Box<dyn Fn(&mut Model, WindowEvent) -> Option<ControlFlow>>>,
    update_callback: Option<Box<dyn Fn(&mut Model, Duration) -> Option<ControlFlow>>>,
    render_callback: Option<Box<dyn Fn(&mut Model, Duration)>>,

    /// The window's title.
    pub title: String,
    /// Whether the window should be fullscreen.
    pub fullscreen: bool,
    /// The window's width if not fullscreen.
    pub width: u32,
    /// The window's height if not fullscreen.
    pub height: u32,
}

impl<Model: 'static> Flow<Model> {
    /// Creates a new Flow designed to handle a specific kind of model.
    ///
    /// This model is instantiated when the Flow is started.
    pub fn new<F: Fn(&Window) -> Model + 'static>(model_init: F) -> Flow<Model> {
        Flow {
            model_init: Box::new(model_init),
            event_callback: None,
            update_callback: None,
            render_callback: None,
            title: "".to_string(),
            fullscreen: false,
            width: 1280,
            height: 720,
        }
    }

    /// Sets the Flow's window event callback.
    pub fn event<F: Fn(&mut Model, WindowEvent) -> Option<ControlFlow> + 'static>(
        &mut self,
        event_callback: F,
    ) {
        self.event_callback = Some(Box::new(event_callback));
    }

    /// Sets the Flow's update callback.
    pub fn update<F: Fn(&mut Model, Duration) -> Option<ControlFlow> + 'static>(
        &mut self,
        update_callback: F,
    ) {
        self.update_callback = Some(Box::new(update_callback));
    }

    /// Sets the Flow's render callback.
    pub fn render<F: Fn(&mut Model, Duration) + 'static>(&mut self, render_callback: F) {
        self.render_callback = Some(Box::new(render_callback));
    }

    /// Starts the Flow's event loop.
    pub fn start(self) -> Result<(), FlowStartError> {
        let event_loop = EventLoop::new();
        let mut builder = WindowBuilder::new().with_title(self.title.clone());

        builder = if self.fullscreen {
            builder.with_fullscreen(
                event_loop
                    .available_monitors()
                    .next()
                    .map(|m| Fullscreen::Borderless(m)),
            )
        } else {
            builder.with_inner_size(PhysicalSize::new(self.width, self.height))
        };

        let window = builder.build(&event_loop)?;

        let mut model = (self.model_init)(&window);
        let mut previous_update = SystemTime::now();
        let mut previous_render = SystemTime::now();

        event_loop.run(move |event, _, control| match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                if let Some(event_callback) = &self.event_callback {
                    if let Some(req) = event_callback(&mut model, event) {
                        *control = req;
                    }
                }
            }
            Event::MainEventsCleared => {
                let now = SystemTime::now();
                let delta = now.duration_since(previous_update).unwrap();
                previous_update = now;

                if let Some(update_callback) = &self.update_callback {
                    if let Some(req) = update_callback(&mut model, delta) {
                        *control = req;
                    }
                }

                if *control != ControlFlow::Exit {
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = SystemTime::now();
                let delta = now.duration_since(previous_render).unwrap();
                previous_render = now;

                if let Some(render_callback) = &self.render_callback {
                    render_callback(&mut model, delta);
                }
            }
            _ => {}
        });
    }
}

#[derive(Debug)]
pub enum FlowStartError {
    OsError(OsError),
}

impl From<OsError> for FlowStartError {
    fn from(e: OsError) -> Self {
        FlowStartError::OsError(e)
    }
}
