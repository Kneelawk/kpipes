mod flow;
mod render;

use flow::Flow;
use futures::executor::block_on;
use render::RenderEngine;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

fn main() {
    let mut flow = Flow::new(KPipes::init);
    flow.event(KPipes::event);
    flow.update(KPipes::update);
    flow.render(KPipes::render);
    flow.width = 1280;
    flow.height = 720;
    flow.title = "KPipes".to_string();

    flow.start().unwrap();
}

struct KPipes {
    renderer: RenderEngine,
}

impl KPipes {
    fn init(window: &Window) -> KPipes {
        KPipes {
            renderer: block_on(RenderEngine::new(window)).unwrap(),
        }
    }

    fn event(&mut self, event: WindowEvent) -> Option<ControlFlow> {
        match event {
            WindowEvent::CloseRequested => Some(ControlFlow::Exit),
            WindowEvent::KeyboardInput { input, .. } => self.keyboard_event(input),
            WindowEvent::Resized(size) => {
                self.renderer.resize(size);
                None
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.renderer.resize(*new_inner_size);
                None
            }
            _ => None,
        }
    }

    fn keyboard_event(&mut self, input: KeyboardInput) -> Option<ControlFlow> {
        match input {
            KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
            } => Some(ControlFlow::Exit),
            _ => None,
        }
    }

    fn update(&mut self) -> Option<ControlFlow> {
        None
    }

    fn render(&mut self) {
        self.renderer.render().unwrap();
    }
}
