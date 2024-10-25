mod config;
mod ui;
mod render;

use std::sync::Arc;

use config::Config;
use winit::{
    event_loop::{ActiveEventLoop, EventLoopBuilder},
    window::WindowAttributes,
};

fn main() {
    let event_loop = EventLoopBuilder::<()>::default().build().unwrap();
    let window = Arc::new(
        event_loop
            .create_window(WindowAttributes::default().with_title("RustSpeak"))
            .unwrap(),
    );
    let mut state =
        pollster::block_on(wgpu_biolerless::StateBuilder::new().window(window).build()).unwrap();
    let config = Config::load();
    event_loop
        .run(move |event, control_flow| {
            let redraw = || {
                let models = screen_sys.tick(&client, &window);
                renderer.render(models);
            };
            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::Resized(size) => {
                            if !state.resize(size) {
                                panic!("Couldn't resize!");
                            } else {
                                renderer.dimensions.set(size.width, size.height);
                            }
                            renderer.rescale_glyphs();
                            redraw();
                        }
                        WindowEvent::Moved(_) => {}
                        WindowEvent::CloseRequested => {
                            control_flow.exit();
                        }
                        WindowEvent::Destroyed => {}
                        WindowEvent::DroppedFile(_) => {}
                        WindowEvent::HoveredFile(_) => {}
                        WindowEvent::HoveredFileCancelled => {}
                        WindowEvent::Focused(_) => {}
                        WindowEvent::KeyboardInput { event, .. } => {
                            screen_sys.press_key(
                                event.physical_key,
                                event.state == ElementState::Pressed,
                            );
                            redraw();
                        }
                        WindowEvent::ModifiersChanged(_) => {}
                        WindowEvent::CursorMoved { position, .. } => {
                            let (width, height) = renderer.dimensions.get();
                            mouse_pos =
                                (position.x / width as f64, 1.0 - position.y / height as f64);
                        }
                        WindowEvent::CursorEntered { .. } => {}
                        WindowEvent::CursorLeft { .. } => {}
                        WindowEvent::MouseWheel { .. } => {}
                        WindowEvent::MouseInput { button, state, .. } => {
                            if button == MouseButton::Left && state == ElementState::Released {
                                screen_sys.on_mouse_click(&client, mouse_pos);
                                redraw();
                            }
                        }
                        WindowEvent::TouchpadPressure { .. } => {}
                        WindowEvent::AxisMotion { .. } => {}
                        WindowEvent::Touch(_) => {}
                        WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            inner_size_writer,
                        } => {
                            if !state.resize((
                                (state.size().1 as f64 * scale_factor) as u32,
                                state.size().1,
                            )) {
                                panic!("Couldn't resize!");
                            }
                            renderer.rescale_glyphs();
                            redraw();
                        }
                        WindowEvent::ThemeChanged(_) => {}
                        WindowEvent::Occluded(_) => {}
                        WindowEvent::RedrawRequested => {
                            // perform redraw
                            redraw();
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent { .. } => {}
                Event::UserEvent(event) => {}
                _ => {}
            }
        })
        .unwrap();
}
