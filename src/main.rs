#[macro_use]
extern crate enum_iterator;

mod direction;
mod flow;
mod render;
mod spaces;

use crate::{
    direction::Direction,
    render::{
        instance::Instance,
        lighting::{Light, Lighting},
    },
    spaces::Spaces,
};
use arrayvec::ArrayVec;
use cgmath::{Matrix4, One, Quaternion, Rad, Rotation3, Vector3};
use enum_iterator::IntoEnumIterator;
use flow::Flow;
use futures::executor::block_on;
use rand::{thread_rng, Rng};
use render::RenderEngine;
use std::{f32::consts::PI, io::Cursor, time::Duration};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

/// How long to wait before causing a pipe to grow.
const GROWTH_DURATION: Duration = Duration::from_millis(50);
/// How many times to try to spawn a new pipe before clearing all the pipes.
const MAX_START_ATTEMPTS: u32 = 3;

const SPACE_WIDTH: usize = 20;
const SPACE_HEIGHT: usize = 20;
const SPACE_DEPTH: usize = 20;

const SINGLE_OBJ: &[u8] = include_bytes!("kpipe-single.obj");
const START_OBJ: &[u8] = include_bytes!("kpipe-start.obj");
const STRAIGHT_OBJ: &[u8] = include_bytes!("kpipe-straight.obj");
const BENT_OBJ: &[u8] = include_bytes!("kpipe-bent.obj");
const END_OBJ: &[u8] = include_bytes!("kpipe-end.obj");

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
    spaces: Spaces,
    time_since_growth: Duration,
    current_color: Vector3<f32>,
    previous_segment: Option<PreviousSegment>,
}

impl KPipes {
    fn init(window: &Window) -> KPipes {
        KPipes {
            renderer: block_on(RenderEngine::new(
                window,
                Lighting::new(
                    [
                        Light::new((-2.0, 3.0, -4.0).into(), 1.0),
                        Light::new((1.0, 2.0, 3.0).into(), 0.6),
                    ],
                    0.2,
                ),
                &mut [
                    Cursor::new(SINGLE_OBJ),
                    Cursor::new(START_OBJ),
                    Cursor::new(STRAIGHT_OBJ),
                    Cursor::new(BENT_OBJ),
                    Cursor::new(END_OBJ),
                ],
                SPACE_WIDTH as u64 * SPACE_HEIGHT as u64 * SPACE_DEPTH as u64,
            ))
            .unwrap(),
            rot: 0.0,
            spaces: Default::default(),
            time_since_growth: Default::default(),
            current_color: random_color(),
            previous_segment: None,
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
                virtual_keycode: Some(VirtualKeyCode::C),
                ..
            } => {
                self.clear_pipes();

                None
            }
            _ => None,
        }
    }

    fn update(&mut self, delta: Duration) -> Option<ControlFlow> {
        // update pipes
        self.time_since_growth += delta;
        if self.time_since_growth > GROWTH_DURATION {
            self.time_since_growth -= GROWTH_DURATION;
            self.grow();
        }

        // update camera
        self.rot += delta.as_secs_f32() * 0.05;

        if self.rot >= PI * 2.0 {
            self.rot -= PI * 2.0;
        }

        let x = self.rot.sin() * 22.0;
        let z = self.rot.cos() * 22.0;

        self.renderer.camera.eye = (x, 15.0, z).into();

        block_on(self.renderer.update_camera()).unwrap();

        None
    }

    fn render(&mut self, _delta: Duration) {
        self.renderer.render().unwrap();
    }

    /// Performs a growth step (either growing the current pipe, starting a new
    /// one, or clearing the screen).
    fn grow(&mut self) {
        if let Some(prev) = self.previous_segment {
            self.grow_existing(prev);
        } else {
            self.new_pipe();
        };
    }

    /// Places a pipe segment connected to an existing pipe, changing existing
    /// pipe models as needed. Will start a new pipe if the current pipe is
    /// boxed in.
    fn grow_existing(&mut self, prev: PreviousSegment) {
        let mut directions = ArrayVec::<[Direction; 6]>::new();

        for direction in Direction::into_enum_iter() {
            if direction.is_offset_legal(prev.location)
                && !self.spaces.get_vec(direction.offset(prev.location))
            {
                directions.push(direction);
            }
        }

        if directions.is_empty() {
            self.new_pipe();
        } else {
            let mut rand = thread_rng();

            let direction: Direction = directions[rand.gen_range(0, directions.len())];
            let location = direction.offset(prev.location);

            match prev.group {
                0 => {
                    self.renderer.remove_instances(0, 1).unwrap();
                    block_on(self.renderer.add_instances(
                        1,
                        &[Instance {
                            color: self.current_color,
                            model: location_matrix(prev.location)
                                * starting_direction_matrix(direction),
                        }],
                    ))
                    .unwrap();
                }
                4 => {
                    self.renderer.remove_instances(4, 1).unwrap();
                    let (rot_matrix, group) = direction_matrix(prev.direction, direction);
                    block_on(self.renderer.add_instances(
                        group,
                        &[Instance {
                            color: self.current_color,
                            model: location_matrix(prev.location) * rot_matrix,
                        }],
                    ))
                    .unwrap();
                }
                _ => unreachable!("Invalid previous group type: {}", prev.group),
            }

            block_on(self.renderer.add_instances(
                4,
                &[Instance {
                    color: self.current_color,
                    model: location_matrix(location) * starting_direction_matrix(direction),
                }],
            ))
            .unwrap();

            self.spaces.set_vec(location);
            self.previous_segment = Some(PreviousSegment {
                direction,
                location,
                group: 4,
            });
        }
    }

    /// Starts growing a new pipe. Will clear the pipes and start over if a
    /// suitable location cannot be found.
    fn new_pipe(&mut self) {
        let mut attempts = 0;

        let location = loop {
            if attempts >= MAX_START_ATTEMPTS {
                self.clear_pipes();

                let location = random_location();
                if self.spaces.get_vec(location) {
                    panic!("Encountered occupied space in cleared board!");
                }
                break location;
            }

            let location = random_location();
            if !self.spaces.get_vec(location) {
                break location;
            }

            attempts += 1;
        };

        self.current_color = random_color();

        block_on(self.renderer.add_instances(
            0,
            &[Instance {
                color: self.current_color,
                model: location_matrix(location),
            }],
        ))
        .unwrap();

        self.spaces.set_vec(location);
        self.previous_segment = Some(PreviousSegment {
            direction: Direction::Up,
            location,
            group: 0,
        });
    }

    /// Clears all the pipes.
    fn clear_pipes(&mut self) {
        self.spaces.clear();
        self.renderer.clear_instances(0);
        self.renderer.clear_instances(1);
        self.renderer.clear_instances(2);
        self.renderer.clear_instances(3);
        self.renderer.clear_instances(4);
        self.previous_segment = None;
    }
}

#[derive(Debug, Copy, Clone)]
struct PreviousSegment {
    direction: Direction,
    location: Vector3<usize>,
    group: usize,
}

/// Generates a random color.
fn random_color() -> Vector3<f32> {
    let mut rand = thread_rng();
    Vector3::new(rand.gen(), rand.gen(), rand.gen())
}

/// Generates a random location within the bounds of the pipe space.
fn random_location() -> Vector3<usize> {
    let mut rand = thread_rng();
    Vector3::new(
        rand.gen_range(0, SPACE_WIDTH),
        rand.gen_range(0, SPACE_HEIGHT),
        rand.gen_range(0, SPACE_DEPTH),
    )
}

/// Converts a location vector into a translation matrix.
fn location_matrix(location: Vector3<usize>) -> Matrix4<f32> {
    Matrix4::from_translation(Vector3::new(
        location.x as f32 - 9.5,
        location.y as f32 - 9.5,
        location.z as f32 - 9.5,
    ))
}

/// Converts a pair of directions into a rotation matrix and pipe type for
/// intermediate pipe segments.
fn direction_matrix(primary: Direction, secondary: Direction) -> (Matrix4<f32>, usize) {
    let (quat, index) = match primary {
        Direction::Up => match secondary {
            Direction::Up => (Quaternion::one(), 2),
            Direction::Down => unreachable!("Up -> Down"),
            Direction::East => (Quaternion::from_angle_y(Rad(PI / 2.0)), 3),
            Direction::West => (Quaternion::from_angle_y(Rad(-PI / 2.0)), 3),
            Direction::South => (Quaternion::one(), 3),
            Direction::North => (Quaternion::from_angle_y(Rad(PI)), 3),
        },
        Direction::Down => match secondary {
            Direction::Up => unreachable!("Down -> Up"),
            Direction::Down => (Quaternion::from_angle_x(Rad(PI)), 2),
            Direction::East => (
                Quaternion::from_angle_y(Rad(-PI / 2.0)) * Quaternion::from_angle_x(Rad(PI)),
                3,
            ),
            Direction::West => (
                Quaternion::from_angle_y(Rad(PI / 2.0)) * Quaternion::from_angle_x(Rad(PI)),
                3,
            ),
            Direction::South => (
                Quaternion::from_angle_y(Rad(PI)) * Quaternion::from_angle_x(Rad(PI)),
                3,
            ),
            Direction::North => (Quaternion::from_angle_x(Rad(PI)), 3),
        },
        Direction::East => match secondary {
            Direction::Up => (
                Quaternion::from_angle_x(Rad(-PI / 2.0)) * Quaternion::from_angle_z(Rad(-PI / 2.0)),
                3,
            ),
            Direction::Down => (
                Quaternion::from_angle_x(Rad(PI / 2.0)) * Quaternion::from_angle_z(Rad(-PI / 2.0)),
                3,
            ),
            Direction::East => (Quaternion::from_angle_z(Rad(-PI / 2.0)), 2),
            Direction::West => unreachable!("East -> West"),
            Direction::South => (Quaternion::from_angle_z(Rad(-PI / 2.0)), 3),
            Direction::North => (
                Quaternion::from_angle_x(Rad(PI)) * Quaternion::from_angle_z(Rad(-PI / 2.0)),
                3,
            ),
        },
        Direction::West => match secondary {
            Direction::Up => (
                Quaternion::from_angle_x(Rad(-PI / 2.0)) * Quaternion::from_angle_z(Rad(PI / 2.0)),
                3,
            ),
            Direction::Down => (
                Quaternion::from_angle_x(Rad(PI / 2.0)) * Quaternion::from_angle_z(Rad(PI / 2.0)),
                3,
            ),
            Direction::East => unreachable!("West -> East"),
            Direction::West => (Quaternion::from_angle_z(Rad(PI / 2.0)), 2),
            Direction::South => (Quaternion::from_angle_z(Rad(PI / 2.0)), 3),
            Direction::North => (
                Quaternion::from_angle_x(Rad(PI)) * Quaternion::from_angle_z(Rad(PI / 2.0)),
                3,
            ),
        },
        Direction::South => match secondary {
            Direction::Up => (
                Quaternion::from_angle_z(Rad(PI)) * Quaternion::from_angle_x(Rad(PI / 2.0)),
                3,
            ),
            Direction::Down => (Quaternion::from_angle_x(Rad(PI / 2.0)), 3),
            Direction::East => (
                Quaternion::from_angle_z(Rad(PI / 2.0)) * Quaternion::from_angle_x(Rad(PI / 2.0)),
                3,
            ),
            Direction::West => (
                Quaternion::from_angle_z(Rad(-PI / 2.0)) * Quaternion::from_angle_x(Rad(PI / 2.0)),
                3,
            ),
            Direction::South => (Quaternion::from_angle_x(Rad(PI / 2.0)), 2),
            Direction::North => unreachable!("South -> North"),
        },
        Direction::North => match secondary {
            Direction::Up => (Quaternion::from_angle_x(Rad(-PI / 2.0)), 3),
            Direction::Down => (
                Quaternion::from_angle_z(Rad(PI)) * Quaternion::from_angle_x(Rad(-PI / 2.0)),
                3,
            ),
            Direction::East => (
                Quaternion::from_angle_z(Rad(-PI / 2.0)) * Quaternion::from_angle_x(Rad(-PI / 2.0)),
                3,
            ),
            Direction::West => (
                Quaternion::from_angle_z(Rad(PI / 2.0)) * Quaternion::from_angle_x(Rad(-PI / 2.0)),
                3,
            ),
            Direction::South => unreachable!("North -> South"),
            Direction::North => (Quaternion::from_angle_x(Rad(-PI / 2.0)), 2),
        },
    };

    (Matrix4::from(quat), index)
}

/// Converts a direction into a rotation matrix for an endpoint pipe segment.
fn starting_direction_matrix(direction: Direction) -> Matrix4<f32> {
    match direction {
        Direction::Up => Matrix4::one(),
        Direction::Down => Matrix4::from_angle_x(Rad(PI)),
        Direction::East => Matrix4::from_angle_z(Rad(-PI / 2.0)),
        Direction::West => Matrix4::from_angle_z(Rad(PI / 2.0)),
        Direction::South => Matrix4::from_angle_x(Rad(PI / 2.0)),
        Direction::North => Matrix4::from_angle_x(Rad(-PI / 2.0)),
    }
}
