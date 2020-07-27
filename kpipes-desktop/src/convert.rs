use kpipes_core::messages::{FlowEvent, FrameSize, KeyCode, KeyState, KeyboardEvent};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
};

/// Used for converting a physical size into something window-system agnostic.
pub trait FromPhysicalSize {
    fn from_physical_size(size: PhysicalSize<u32>) -> Self;
}

/// Used for converting an element state into something window-system agnostic.
pub trait FromElementState {
    fn from_element_state(state: ElementState) -> Self;
}

/// Used for converting a virtual key code into something window-system
/// agnostic.
pub trait FromVirtualKeyCode {
    fn from_virtual_key_code(code: VirtualKeyCode) -> Self;
}

/// Used for converting a keyboard input into something window-system agnostic.
pub trait FromKeyboardInput {
    fn from_keyboard_input(input: KeyboardInput) -> Self;
}

/// Used for converting a window event into something window-system agnostic.
pub trait FromWindowEvent {
    fn from_window_event(e: WindowEvent) -> Self;
}

impl FromPhysicalSize for FrameSize {
    fn from_physical_size(size: PhysicalSize<u32>) -> Self {
        FrameSize {
            width: size.width,
            height: size.height,
        }
    }
}

impl FromElementState for KeyState {
    fn from_element_state(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => KeyState::Pressed,
            ElementState::Released => KeyState::Released,
        }
    }
}

impl FromVirtualKeyCode for KeyCode {
    fn from_virtual_key_code(code: VirtualKeyCode) -> Self {
        match code {
            VirtualKeyCode::Escape => KeyCode::Escape,
            VirtualKeyCode::C => KeyCode::C,
            _ => KeyCode::Other,
        }
    }
}

impl FromKeyboardInput for KeyboardEvent {
    fn from_keyboard_input(input: KeyboardInput) -> Self {
        KeyboardEvent {
            state: KeyState::from_element_state(input.state),
            virtual_keycode: match input.virtual_keycode {
                Some(code) => Some(KeyCode::from_virtual_key_code(code)),
                None => None,
            },
        }
    }
}

impl FromWindowEvent for FlowEvent {
    fn from_window_event(e: WindowEvent<'_>) -> Self {
        match e {
            WindowEvent::Resized(size) => FlowEvent::Resized(FrameSize::from_physical_size(size)),
            WindowEvent::CloseRequested => FlowEvent::CloseRequested,
            WindowEvent::KeyboardInput {
                input,
                is_synthetic,
                ..
            } => FlowEvent::KeyboardInput {
                input: KeyboardEvent::from_keyboard_input(input),
                is_synthetic,
            },
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => FlowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size: FrameSize::from_physical_size(*new_inner_size),
            },
            _ => FlowEvent::Other,
        }
    }
}
