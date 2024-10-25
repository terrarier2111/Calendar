mod config;
mod ui;
mod render;
mod screen_sys;

use std::sync::{Arc, Mutex, RwLock};

use config::Config;
use glyphon::{Attrs, AttrsOwned, Color, Shaping};
use render::{ctx, GlyphId, GlyphInfo, Renderer, UiCtx};
use screen_sys::Screen;
use ui::{Container, TextBox};
use winit::{
    event::{Event, WindowEvent}, event_loop::{ActiveEventLoop, EventLoopBuilder}, window::WindowAttributes
};

fn main() {
    let event_loop = EventLoopBuilder::<()>::default().build().unwrap();
    let window = Arc::new(
        event_loop
            .create_window(WindowAttributes::default().with_title("RustSpeak"))
            .unwrap(),
    );
    let state =
        Arc::new(pollster::block_on(wgpu_biolerless::StateBuilder::new().window(window.clone()).build()).unwrap());
    let renderer = Arc::new(Renderer::new(state.clone(), &window).unwrap());
    let config = Config::load();
    render::init(Arc::new(UiCtx {
        renderer: renderer.clone(),
        window: window.clone(),
    }));
    let screen_sys = Arc::new(screen_sys::ScreenSystem::new());
    screen_sys.push_screen(Box::new(DefaultScreen::new()));
    event_loop
        .run(move |event, control_flow| {
            let redraw = || {
                let models = screen_sys.tick(&Arc::new(()), &window);
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
                        WindowEvent::KeyboardInput { event, .. } => {}
                        WindowEvent::ModifiersChanged(_) => {}
                        WindowEvent::CursorMoved { position, .. } => {}
                        WindowEvent::CursorEntered { .. } => {}
                        WindowEvent::CursorLeft { .. } => {}
                        WindowEvent::MouseWheel { .. } => {}
                        WindowEvent::MouseInput { button, state, .. } => {}
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

#[derive(Clone)]
struct DefaultScreen {
    render: Arc<Container>,
}

impl DefaultScreen {

    fn new() -> Self {
        Self {
            render: Arc::new(Container::new()),
        }
    }

}

impl Screen for DefaultScreen {
    fn on_active(&mut self, _ctx: &Arc<screen_sys::AppCtx>) {
        let glyph = ctx().renderer.add_glyph(GlyphInfo { in_bounds_off: (0.0, 0.0), size: (0.2, 0.2), text: "test".to_string(), attrs: AttrsOwned::new(Attrs::new()), shaping: Shaping::Basic, color: Color(u8::MAX as u32), scale: 1.0, x_offset: 0.0, y_offset: 0.0 });
        self.render.add(Arc::new(RwLock::new(Box::new(TextBox {
            pos: (0.0, 0.0),
            width: 0.1,
            height: 0.1,
            coloring: [ui::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }; 6],
            texts: vec![glyph],
        }))));
    }

    fn on_deactive(&mut self, _ctx: &Arc<screen_sys::AppCtx>) {
        self.render.clear();
    }

    fn tick(&mut self, _ctx: &Arc<screen_sys::AppCtx>) {
        
    }

    fn container(&self) -> &Arc<ui::Container> {
        &self.render
    }

    fn clone_screen(&self) -> Box<dyn Screen> {
        Box::new(self.clone())
    }
}
