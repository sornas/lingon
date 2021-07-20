//! The easiest way to get the user to make decisions.
//!
//! A small example:
//! ```ignore
//! #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
//! enum Name {
//!     Left,
//!     Right,
//!     // ...
//! }
//! fn main() {
//!     let input = input::InputManager::new(sdl);
//!     input.bind(Name::Left, input::Device::Key(input::KeycodeA));
//!
//!     // main loop
//!     loop {
//!         input.poll(sdl);
//!         // ...
//!         if input.pressed(Name::Left) {
//!             player_jump();
//!         }
//!     }
//! }
//! ```
//! Here we bind the "A" key to the input "Left", and tell the player to jump when the
//! button is first pressed.
//!
//! For this to work the InputManager needs to be passed around everywhere it's used,
//! and to use the rumble feature it needs to be mutable.

pub use sdl2::controller::{Axis, Button};
pub use sdl2::keyboard::Keycode;
pub use sdl2::mouse::MouseButton;

use luminance_sdl2::sdl2;
use sdl2::{GameControllerSubsystem, Sdl};
use sdl2::controller::GameController;
use sdl2::event::{Event, WindowEvent};
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;

/// All the different kinds of input devices we can listen to.
#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Device {
    /// The magic quit event, when the window is closed.
    Quit,
    Key(Keycode),
    Mouse(MouseButton),
    Button(u32, Button),
    Axis(u32, Axis),
}

#[derive(Copy, Clone, Debug)]
enum KeyState {
    Down(usize),
    Up(usize),
    Analog(f32),
}

/// The one stop shop for all things input!
pub struct InputManager<T> {
    frame: usize,
    controllers: GameControllerSubsystem,
    physical_inputs: HashMap<Device, T>,
    virtual_inputs: HashMap<T, KeyState>,
    opened_controllers: HashMap<u32, GameController>,
    mouse: [i32; 2],
    /// Since the last call to [InputManager::poll].
    mouse_rel: [i32; 2],
    text_input_enabled: bool,
    text_input_events: Vec<Keycode>,
}

/// [i32::MIN, i32::MAX] -> [-1.0, 1.0)
fn remap(value: i16) -> f32 {
    // MIN has a larger absolute value,
    // this garantees that the values in [-1, 1).
    let value = -(value as f32) / (i16::MIN as f32);
    // Arbitrarily chosen
    const DEADZONE: f32 = 0.10;
    if value.abs() < DEADZONE {
        0.0
    } else {
        value / (1.0 - DEADZONE)
    }
}

/// When an analog signal becomes digital.
const TRIGGER_LIMIT: f32 = 0.1;

impl<T> InputManager<T>
where
    T: Clone + Hash + Eq,
{
    pub fn new(sdl: &Sdl) -> Self {
        let controllers = sdl.game_controller().unwrap();
        controllers.set_event_state(true);

        Self {
            physical_inputs: HashMap::new(),
            virtual_inputs: HashMap::new(),
            frame: 0,
            controllers: controllers.clone(),
            opened_controllers: HashMap::new(),
            mouse: [0, 0],
            mouse_rel: [0, 0],
            text_input_enabled: false,
            text_input_events: Vec::new(),
        }
    }

    /// Creates a new binding to listen to.
    pub fn bind(&mut self, device: Device, name: T) {
        self.physical_inputs.insert(device, name.clone());
        self.virtual_inputs.insert(name, KeyState::Up(0));
    }

    /// Check if the input is down this frame.
    pub fn down(&self, name: T) -> bool {
        match self.virtual_inputs.get(&name) {
            Some(KeyState::Down(_)) => true,
            Some(KeyState::Up(_)) => false,
            Some(KeyState::Analog(v)) => v.abs() > TRIGGER_LIMIT,
            None => {
                // TODO(ed): I don't like this... but it's here now.
                false
            }
        }
    }

    /// Check if the input is inactive.
    pub fn up(&self, name: T) -> bool {
        match self.virtual_inputs.get(&name) {
            Some(KeyState::Down(_)) => false,
            Some(KeyState::Up(_)) => true,
            Some(KeyState::Analog(v)) => v.abs() < TRIGGER_LIMIT,
            None => {
                // TODO(ed): I don't like this... but it's here now.
                false
            }
        }
    }

    /// Check if the input is pressed this frame.
    pub fn pressed(&self, name: T) -> bool {
        match self.virtual_inputs.get(&name) {
            Some(KeyState::Down(frame)) => self.frame == *frame,
            _ => {
                // TODO(ed): I don't like this... but it's here now.
                false
            }
        }
    }

    /// Check if the input was released this frame.
    pub fn released(&self, name: T) -> bool {
        match self.virtual_inputs.get(&name) {
            Some(KeyState::Up(frame)) => self.frame == *frame,
            _ => {
                // TODO(ed): I don't like this... but it's here now.
                false
            }
        }
    }

    /// Returns the inputs as analog signals.
    pub fn value(&self, name: T) -> f32 {
        match self.virtual_inputs.get(&name) {
            Some(KeyState::Up(_)) => 0.0,
            Some(KeyState::Down(_)) => 1.0,
            Some(KeyState::Analog(v)) => *v,
            _ => {
                // TODO(ed): I don't like this... but it's here now.
                0.0
            }
        }
    }

    /// Shake that controller!
    pub fn rumble(&mut self, controller: u32, lo: f32, hi: f32, time: f32) -> Result<(), ()> {
        if let Some(controller) = self.opened_controllers.get_mut(&controller) {
            let lo = (lo * (u16::MAX as f32)) as u16;
            let hi = (hi * (u16::MAX as f32)) as u16;
            controller
                .set_rumble(lo, hi, (time * 1000.0) as u32)
                .unwrap();
            Ok(())
        } else {
            Err(())
        }
    }

    /// Returns the current mouse position.
    pub fn mouse(&self) -> (i32, i32) {
        (self.mouse[0], self.mouse[1])
    }

    /// Returns the relative mouse movement since the last call to
    /// [InputManager::poll].
    pub fn mouse_rel(&self) -> (i32, i32) {
        (self.mouse_rel[0], self.mouse_rel[1])
    }

    pub fn set_text_input_enabled(&mut self, enabled: bool) {
        self.text_input_enabled = enabled;
    }

    pub fn text_input_update(&mut self, s: &mut String) {
        for keycode in std::mem::take(&mut self.text_input_events) {
            match keycode {
                Keycode::Backspace => { s.pop(); }
                c => if let Some(c) = (c as i32).try_into().ok().and_then(char::from_u32) {
                    s.push(c);
                }
            }
        }
    }

    /// Update the state of the input.
    pub fn poll(&mut self, sdl: &sdl2::Sdl) {
        self.frame += 1;
        self.mouse_rel = [0, 0];
        let frame = self.frame;
        for event in sdl.event_pump().unwrap().poll_iter() {
            let (input, down) = match event {
                Event::Quit { .. } => (Device::Quit, KeyState::Down(frame)),
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                } => (Device::Quit, KeyState::Down(frame)),
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if repeat {
                        continue;
                    }
                    if self.text_input_enabled {
                        self.text_input_events.push(keycode);
                    }
                    (Device::Key(keycode), KeyState::Down(frame))
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => (Device::Key(keycode), KeyState::Up(frame)),
                Event::ControllerDeviceAdded { which, .. } => {
                    let controller = self.controllers.open(which).unwrap();
                    self.opened_controllers.insert(which, controller);
                    continue;
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    self.opened_controllers.remove(&which).unwrap();
                    continue;
                }
                Event::ControllerAxisMotion {
                    which, axis, value, ..
                } => {
                    let value = remap(value);
                    (Device::Axis(which, axis), KeyState::Analog(value))
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    (Device::Button(which, button), KeyState::Down(frame))
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    (Device::Button(which, button), KeyState::Up(frame))
                }
                Event::MouseButtonDown { mouse_btn, .. } => {
                    (Device::Mouse(mouse_btn), KeyState::Down(frame))
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    (Device::Mouse(mouse_btn), KeyState::Up(frame))
                }
                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    self.mouse = [x, y];
                    self.mouse_rel[0] += xrel;
                    self.mouse_rel[1] += yrel;
                    continue;
                }
                _ => {
                    continue;
                }
            };

            if let Some(slot) = self.physical_inputs.get(&input) {
                self.virtual_inputs.insert(slot.clone(), down);
            }
        }
    }
}
