mod flow;

use flow::Flow;
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

struct KPipes;

impl KPipes {
    fn init(_window: &Window) -> KPipes {
        KPipes
    }

    fn event(&mut self, event: WindowEvent) -> Option<ControlFlow> {
        match event {
            WindowEvent::CloseRequested => Some(ControlFlow::Exit),
            WindowEvent::KeyboardInput { input, .. } => self.keyboard_event(input),
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

    fn render(&mut self) {}
}
