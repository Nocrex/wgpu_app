use winit::event::Event;

use crate::{
    io::{keyboard::Keyboard, mouse::Mouse},
    WgpuState,
};

pub struct Context {
    pub wgpu_state: WgpuState,

    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub block_gui_input: bool,
    pub block_gui_tab_input: bool,
}

/// `Context` stores some useful things you might want to use in your app, including input from a Keyboard and Mouse,
/// the Display to render to and an instead of EguiGlium for all your gui needs!
impl Context {
    pub fn new(wgpu_state: WgpuState) -> Context {
        Context {
            wgpu_state,

            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
            block_gui_input: false,
            block_gui_tab_input: false,
        }
    }

    /// This function is automatically called in the application loop, you shouldn't need to call it yourself
    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            _ => {
                self.keyboard.handle_event(event);
                self.mouse.handle_event(event);
            }
        }
    }

    // Attempts to restrict the mouse movement to inside the window
    //
    // # Errors:
    // This function can fail for a number of reasons, a common one might be that the mouse is already grabbed by another application or the OS
    // this does happen occasionally such as if the user grabs the title bar of the window to drag it around on many Linux machines
    // so be a little careful on when you try to grab the mouse, such as when receiving focus.
    // pub fn set_mouse_grabbed(&self, grabbed: bool) -> Result<(), ExternalError> {
    //     let gl_win = self.dis.gl_window();
    //     let win = gl_win.window();
    //
    //     win.set_cursor_grab(grabbed)
    // }

    // Sets the mouse visible or invisible
    // pub fn set_mouse_visible(&self, visible: bool) {
    //     let gl_win = self.dis.gl_window();
    //     let win = gl_win.window();
    //
    //     win.set_cursor_visible(visible);
    // }
}
