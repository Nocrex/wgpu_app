use context::{Context, EguiManager, WgpuState};

pub mod context;
pub mod io;
pub mod timer;
pub mod utils;

use egui_winit::winit::{
    event::{KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
};
pub use timer::Timer;

use winit::{
    event::{self, Event},
    window::WindowBuilder,
};

/// Implement this trait to run it with `run` or `run_with_context`!
pub trait Application {
    /// This function is called after everything is setup but before the first frame is rendered
    fn init(&mut self, ctx: &mut Context);
    /// Called every frame to give the application a chance to update and render, the timer provides information like the time since the last frame and the current frame rate
    fn update(&mut self, t: &Timer, ctx: &mut Context) -> Result<(), wgpu::SurfaceError>;
    /// Called when the window is requested to close
    fn close(&mut self, ctx: &Context);
    /// Called a number of times between each frame with all new incoming events for the application
    fn handle_event(&mut self, ctx: &mut Context, event: &Event<()>);
}

/// Create and run a window for this application
///
/// # Arguments
///
/// * `mut app: Application` - the application you want to run with winit and Wgpu
/// * `wb: WindowBuilder` - Settings on how the window should be shaped/sized/positioned/resizable etc
pub fn run<A: 'static + Application>(app: A, wb: WindowBuilder) {
    let event_loop = winit::event_loop::EventLoopBuilder::<()>::with_user_event().build();
    let window = wb.build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: Default::default(),
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);

    // Shader code assumes an sRGB surface texture. Using a different
    // one will result all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.describe().srgb)
        .next()
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let wgpu_state = WgpuState {
        surface,
        device,
        queue,
        config,
        size,
        window,
    };

    let egui = EguiManager::new(&wgpu_state.device, &event_loop);

    let ctx = Context::new(wgpu_state, egui);

    run_with_context(app, ctx, event_loop);
}

/// Run a wgpu_app `Application` with a provided Context and EventLoop (usually obtained from `create`)
///
/// # Arguments
///
/// * `mut app: Application` - the application you want to run
/// * `mut context: Context` - A wgpu_app Context containing a Display, Egui object and io managers
/// * `event_loop: EventLoop<()>` - The EventLoop for the window
pub fn run_with_context<A: 'static + Application>(
    mut app: A,
    mut context: Context,
    event_loop: EventLoop<()>,
) {
    let mut t = Timer::new();

    t.reset();
    event_loop.run(move |ev, _, control_flow| {
        // Handle our own events
        let mut events_cleared = false;

        match &ev {
            Event::MainEventsCleared => {
                events_cleared = true;
            }
            Event::NewEvents(cause) => match cause {
                event::StartCause::Init => {
                    app.init(&mut context);
                }
                _ => {}
            },
            Event::WindowEvent {
                window_id: _,
                event: event::WindowEvent::CloseRequested,
            } => {
                app.close(&context);
                *control_flow = ControlFlow::Exit;
            }
            _ => {
                context.handle_event(&ev);
                app.handle_event(&mut context, &ev);
            }
        }

        if !events_cleared {
            return;
        }

        // Update
        match t.go() {
            None => {}
            Some(_) => {
                match app.update(&t, &mut context) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        context.wgpu_state.resize(context.wgpu_state.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Err(e) => log::error!("{:?}", e),
                }

                context.mouse.next_frame();
                context.keyboard.next_frame();
            }
        }
    });
}
