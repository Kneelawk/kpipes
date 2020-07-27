/// Describes the size of a texture or window frame.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameSize {
    pub width: u32,
    pub height: u32,
}

/// Describes whether a key was pressed or released.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Describes the key codes important to the KPipes engine.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum KeyCode {
    Escape,
    C,
    Other,
}

/// Describes a keyboard input event.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct KeyboardEvent {
    pub state: KeyState,
    pub virtual_keycode: Option<KeyCode>,
}

/// Used to communicate application events to the KPipes engine.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum FlowEvent {
    Resized(FrameSize),
    CloseRequested,
    KeyboardInput {
        input: KeyboardEvent,
        is_synthetic: bool,
    },
    ScaleFactorChanged {
        scale_factor: f64,
        new_inner_size: FrameSize,
    },
    Other,
}

/// Used by the KPipes engine for communicating requests for controlling the
/// flow of the application.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum FlowControl {
    Exit,
    None,
}
