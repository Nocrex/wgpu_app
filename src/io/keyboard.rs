use egui_winit::winit::event::{ElementState, Event, KeyboardInput, WindowEvent};
use winit::event::VirtualKeyCode;

use std::collections::HashMap;

pub struct Keyboard {
    keys: HashMap<VirtualKeyCode, bool>,
    this_frame: HashMap<VirtualKeyCode, bool>,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            keys: HashMap::new(),
            this_frame: HashMap::new(),
        }
    }

    fn press(&mut self, key: VirtualKeyCode) {
        self.keys.insert(key, true);
        self.this_frame.insert(key, true);
    }

    fn release(&mut self, key: VirtualKeyCode) {
        self.keys.insert(key, false);
        self.this_frame.insert(key, true);
    }

    /// This function is called automatically in the application loop, you shouldn't be calling this yourself.
    pub fn handle_event(&mut self, event: &Event<()>) {
        if let Event::WindowEvent {
            window_id: _,
            event:
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input:
                        KeyboardInput {
                            scancode: _,
                            state,
                            virtual_keycode: Some(key),
                            modifiers: _,
                        },
                    is_synthetic: _,
                },
        } = event
        {
            if state == &ElementState::Pressed {
                self.press(*key);
            } else {
                self.release(*key);
            }
        }
    }

    /// Returns if this key was pressed down on this frame
    pub fn pressed_this_frame(&self, key: &VirtualKeyCode) -> bool {
        match self.keys.get(key) {
            None | Some(false) => false,
            Some(true) => match self.this_frame.get(key) {
                None | Some(false) => false,
                Some(true) => true,
            },
        }
    }

    /// Returns if this key was released on this frame
    pub fn released_this_frame(&self, key: &VirtualKeyCode) -> bool {
        match self.keys.get(key) {
            Some(true) => false,
            None | Some(false) => match self.this_frame.get(key) {
                None | Some(false) => false,
                Some(true) => true,
            },
        }
    }

    /// Returns if the key is currently held down
    pub fn is_pressed(&self, key: &VirtualKeyCode) -> bool {
        match self.keys.get(key) {
            None | Some(false) => false,
            Some(true) => true,
        }
    }

    /// Resets the Keyboard for the next frame, this function is called automatically so you shouldn't need to call this function yourself.
    pub fn next_frame(&mut self) {
        self.this_frame.clear();
    }
}
