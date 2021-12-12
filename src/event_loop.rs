use {
    crate::{
        octree::{Octant, Octree},
        render_state::RenderState,
        types::Point,
    },
    futures::executor,
    log::{error, warn},
    winit::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
};

pub fn start(octree: &mut Octree) -> Result<(), Box<dyn std::error::Error>> {
    let octants: Vec<Octant> = octree.clone().into_iter().collect();
    let points: Vec<Point> = octree.clone().into_iter().filter_map(|o| o.feature).collect();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let mut render_state = executor::block_on(RenderState::new(&window));
    let mut last_render_time = std::time::Instant::now();

    render_state.set_faces_model(octree.extract_faces());
    render_state.set_octree_model(octants);
    render_state.set_points_model(points);

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            // Emitted after MainEventsCleared when a window should be redrawn.
            // called when window is invalidated (ex: resize) or when explicitly requested by
            // `Window::request_redraw`
            let now = std::time::Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;
            render_state.update(dt);
            match render_state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("render error: {:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // Emitted when all of the event loop’s input events have been processed and redraw
            // processing is about to begin. Do stuff like update state, calculation, etc... here
            window.request_redraw();
        }
        Event::DeviceEvent { ref event, device_id: _ } => {
            render_state.device_input(event);
        }
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { .. } => {
                render_state.input(event);
            }
            WindowEvent::Resized(physical_size) => {
                render_state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                render_state.resize(**new_inner_size);
            }
            evt => warn!("unhandled window event: {:?}", evt),
        },
        Event::RedrawEventsCleared => {
            // Emitted after all RedrawRequested events
            // do cleanup after rendering here if needed.
        }
        Event::NewEvents(_) | Event::Suspended | Event::Resumed | Event::UserEvent(_) => {}
        Event::WindowEvent { .. } => error!("bad window_id"),
        Event::LoopDestroyed => {}
    })
}
