use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::{native_pixels_per_point, screen_size_in_pixels, EventResponse};
use wgpu::SurfaceTexture;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopWindowTarget,
};

use crate::{
    io::{keyboard::Keyboard, mouse::Mouse},
    WgpuState,
};

pub struct Context {
    pub wgpu_state: WgpuState,
    pub egui: EguiManager,

    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub block_gui_input: bool,
    pub block_gui_tab_input: bool,
}

pub struct EguiManager {
    pub renderer: egui_wgpu::Renderer,
    pub state: egui_winit::State,
    pub ctx: egui::Context,
}

/// `Context` stores some useful things you might want to use in your app, including input from a Keyboard and Mouse,
/// the Display to render to and an instead of EguiGlium for all your gui needs!
impl Context {
    pub fn new(wgpu_state: WgpuState, egui: EguiManager) -> Context {
        Context {
            wgpu_state,
            egui,

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

    // pub fn get_screen_descriptor(&self) -> ScreenDescriptor {
    //     ScreenDescriptor { size_in_pixels: , pixels_per_point: () }
    // }

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

impl EguiManager {
    /// Setup everything required to render Egui
    pub fn new<T>(device: &wgpu::Device, event_loop: &EventLoopWindowTarget<T>) -> EguiManager {
        EguiManager {
            renderer: egui_wgpu::Renderer::new(
                device,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                None,
                1,
            ),
            state: egui_winit::State::new(event_loop),
            ctx: egui::Context::default(),
        }
    }

    /// Update egui state
    pub fn on_event(&mut self, event: &WindowEvent<'_>) -> EventResponse {
        self.state.on_event(&self.ctx, event)
    }

    /// Render the Egui gui built in `run_ui` to the `output` texture.
    pub fn render<'rp>(
        &'rp mut self,
        wgpu_state: &mut WgpuState,
        output: &SurfaceTexture,
        run_ui: impl FnOnce(&egui::Context),
    ) {
        let run_output = self
            .ctx
            .run(self.state.take_egui_input(&wgpu_state.window), run_ui);
        let screen_size = screen_size_in_pixels(&wgpu_state.window);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [screen_size.x as u32, screen_size.y as u32],
            pixels_per_point: native_pixels_per_point(&wgpu_state.window),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            wgpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Egui command encoder"),
                });

        let clipped_primitives = self.ctx.tessellate(run_output.shapes);
        let user_cmd_bufs = {
            for (id, image_delta) in &run_output.textures_delta.set {
                self.renderer.update_texture(
                    &wgpu_state.device,
                    &wgpu_state.queue,
                    *id,
                    image_delta,
                );
            }

            self.renderer.update_buffers(
                &wgpu_state.device,
                &wgpu_state.queue,
                &mut encoder,
                &clipped_primitives,
                &screen_descriptor,
            )
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        for id in &run_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        let encoded = encoder.finish();
        wgpu_state
            .queue
            .submit(user_cmd_bufs.into_iter().chain(std::iter::once(encoded)));
    }
}
