mod flow;
mod render;

use crate::render::instance::Instance;
use cgmath::Matrix4;
use flow::Flow;
use futures::executor::block_on;
use render::RenderEngine;
use std::time::Duration;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

fn main() {
    env_logger::init();

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
    rot: f32,
    instance: u32,
}

impl KPipes {
    fn init(window: &Window) -> KPipes {
        KPipes {
            renderer: block_on(RenderEngine::new(window)).unwrap(),
            rot: 0.0,
            instance: 0,
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
            KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Space),
                ..
            } => {
                let instance = Instance {
                    color: (self.instance as f32 / 4.0, 0.1, 0.2).into(),
                    model: Matrix4::from_scale(0.5)
                        * Matrix4::from_translation((0.0, self.instance as f32, 0.0).into()),
                };

                if block_on(self.renderer.add_instance(instance)).is_ok() {
                    self.instance += 1;
                } else {
                    self.renderer.clear_instances();
                    self.instance = 0;
                }

                None
            }
            _ => None,
        }
    }

    fn update(&mut self, delta: Duration) -> Option<ControlFlow> {
        self.rot += delta.as_secs_f32() * 0.5;

        let x = self.rot.sin() * 5.0;
        let z = self.rot.cos() * 5.0;

        self.renderer.camera.eye = (x, 5.0, z).into();

        block_on(self.renderer.update_camera()).unwrap();

        None
    }

    fn render(&mut self, _delta: Duration) {
        self.renderer.render().unwrap();
    }
}
