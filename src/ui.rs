use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use atomic_float::AtomicF64;

use crate::render::{ctx, GlyphId, Model, Vertex};

pub type AppCtx = ();

pub trait Component: Send + Sync {
    fn build_model(&self) -> Model;

    // fn is_inbounds(&self, pos: (f32, f32)) -> bool; // FIXME: is this one better?

    fn do_render(&self, _client: &Arc<AppCtx>) {}

    fn pos(&self) -> (f32, f32);

    fn dims(&self) -> (f32, f32);

    fn on_click(&mut self, client: &Arc<AppCtx>);

    fn on_scroll(&mut self, client: &Arc<AppCtx>);

    fn on_hover(&mut self, client: &Arc<AppCtx>, mode: HoverMode);
}

#[derive(Copy, Clone)]
pub enum HoverMode {
    Enter,
    Exit,
}

pub struct UIComponent {
    inner: Arc<InnerUIComponent>,
}

impl UIComponent {
    pub fn build_model(&self) -> Model {
        self.inner.build_model()
    }

    pub fn on_mouse_click(&self, client: &Arc<AppCtx>) {
        self.inner.inner.write().unwrap().on_click(client);
        self.inner.make_dirty();
    }

    pub fn is_inbounds(&self, pos: (f32, f32)) -> bool {
        let inner = self.inner.inner.read().unwrap();
        let dims = inner.dims();
        let inner_pos = inner.pos();
        // println!("pos: {:?}", pos);
        let bounds = (inner_pos.0 + dims.0, inner_pos.1 + dims.1);
        // println!("higher than comp start ({:?}): {}", inner_pos, (pos.0 >= inner_pos.0 && pos.1 >= inner_pos.1));
        // println!("lower than comp end: ({:?}): {}", bounds, (pos.0 <= bounds.0 && pos.1 <= bounds.1));
        (pos.0 >= inner_pos.0 && pos.1 >= inner_pos.1) && (pos.0 <= bounds.0 && pos.1 <= bounds.1)
    }
}

pub struct InnerUIComponent {
    inner: Arc<RwLock<Box<dyn Component>>>, // FIXME: should we prefer a Mutex over a Rwlock?
    precomputed_model: Mutex<Model>,
    dirty: AtomicBool,
}

impl InnerUIComponent {
    fn build_model(&self) -> Model {
        if self.dirty.compare_exchange(true, false, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
            let model = self.inner.write().unwrap().build_model();
            *self.precomputed_model.lock().unwrap() = model.clone();
            model
        } else {
            self.precomputed_model.lock().unwrap().clone()
        }
    }

    pub fn make_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn into_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Default)]
pub struct ScrollData {
    min_y: AtomicF64,
    max_y: AtomicF64,
    min_x: AtomicF64,
    max_x: AtomicF64,
    offset_x: AtomicF64,
    offset_y: AtomicF64,
}

#[derive(Default)]
pub struct Container {
    components: RwLock<Vec<UIComponent>>,
    scroll_data: ScrollData, // FIXME: use this for scroll sliders
}

impl Container {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(self: &Arc<Self>, component: Arc<RwLock<Box<dyn Component>>>) {
        let model = component.read().unwrap().build_model();
        self.components.write().unwrap().push(UIComponent {
            inner: Arc::new(InnerUIComponent {
                inner: component,
                precomputed_model: Mutex::new(model),
                dirty: AtomicBool::new(false),
            }),
        });
    }

    pub fn clear(&self) {
        self.components.write().unwrap().clear();
    }

    pub fn build_models(&self, ctx: &Arc<AppCtx>) -> Vec<Model> {
        let mut models = vec![];
        for component in self.components.read().unwrap().iter() {
            models.push(component.build_model());
            component.inner.inner.read().unwrap().do_render(ctx);
        }
        models
    }

    pub fn on_mouse_click(&self, ctx: &Arc<AppCtx>, pos: (f64, f64)) {
        for component in self.components.read().unwrap().iter() {
            if component.is_inbounds((pos.0 as f32, pos.1 as f32)) { // FIXME: switch to using f64 instead!
                component.on_mouse_click(ctx);
                return;
            }
        }
    }
}

pub struct Button<T: Send + Sync = ()> {
    pub inner_box: TextBox,
    pub data: T,
    pub on_click: Arc<Box<dyn Fn(&mut Button<T>, &Arc<AppCtx>) + Send + Sync>>,
}

impl<T: Send + Sync> Component for Button<T> {
    fn build_model(&self) -> Model {
        self.inner_box.build_model()
    }

    fn do_render(&self, client: &Arc<AppCtx>) {
        self.inner_box.do_render(client)
    }

    fn pos(&self) -> (f32, f32) {
        self.inner_box.pos()
    }

    fn dims(&self) -> (f32, f32) {
        self.inner_box.dims()
    }

    fn on_click(&mut self, ctx: &Arc<AppCtx>) {
        let func = self.on_click.clone();
        func(self, ctx);
    }

    fn on_scroll(&mut self, _ctx: &Arc<AppCtx>) {}

    fn on_hover(&mut self, _ctx: &Arc<AppCtx>, _mode: HoverMode) {}
}

pub struct ColorBox {
    pub pos: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub coloring: [Color; 6],
}

impl Component for ColorBox {
    fn build_model(&self) -> Model {
        let (x_off, y_off) = ((2.0 * self.pos.0), (2.0 * self.pos.1));
        let vertices = [
            [-1.0 + x_off, -1.0 + y_off],
            [2.0 * self.width - 1.0 + x_off, -1.0 + y_off],
            [
                2.0 * self.width - 1.0 + x_off,
                2.0 * self.height - 1.0 + y_off,
            ],
            [-1.0 + x_off, -1.0 + y_off],
            [-1.0 + x_off, 2.0 * self.height - 1.0 + y_off],
            [
                2.0 * self.width - 1.0 + x_off,
                2.0 * self.height - 1.0 + y_off,
            ],
        ];
        let vertices = {
            let mut ret = Vec::with_capacity(6);
                for (i, pos) in vertices.into_iter().enumerate() {
                    ret.push(Vertex::GenericColor {
                        pos,
                        color: self.coloring[i].into_array(),
                    });
                }
                ret
        };
        Model {
            vertices,
        }
    }

    fn pos(&self) -> (f32, f32) {
        self.pos
    }

    fn dims(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn on_click(&mut self, _client: &Arc<AppCtx>) {}

    fn on_scroll(&mut self, _client: &Arc<AppCtx>) {}

    fn on_hover(&mut self, _client: &Arc<AppCtx>, _mode: HoverMode) {}
}

pub struct TextBox {
    pub pos: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub coloring: [Color; 6],
    pub texts: Vec<GlyphId>,
}

impl Component for TextBox {
    fn build_model(&self) -> Model {
        // FIXME: do we have to rebuild the texts? - and if so, how?
        let (x_off, y_off) = ((2.0 * self.pos.0), (2.0 * self.pos.1));
        let vertices = [
            [-1.0 + x_off, -1.0 + y_off],
            [2.0 * self.width - 1.0 + x_off, -1.0 + y_off],
            [
                2.0 * self.width - 1.0 + x_off,
                2.0 * self.height - 1.0 + y_off,
            ],
            [-1.0 + x_off, -1.0 + y_off],
            [-1.0 + x_off, 2.0 * self.height - 1.0 + y_off],
            [
                2.0 * self.width - 1.0 + x_off,
                2.0 * self.height - 1.0 + y_off,
            ],
        ];
        let vertices = {
            let mut ret = Vec::with_capacity(6);
                for (i, pos) in vertices.into_iter().enumerate() {
                    ret.push(Vertex::GenericColor {
                        pos,
                        color: self.coloring[i].into_array(),
                    });
                }
                ret
        };
        Model {
            vertices,
        }
    }

    fn do_render(&self, _client: &Arc<AppCtx>) {}

    fn pos(&self) -> (f32, f32) {
        self.pos
    }

    fn dims(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn on_click(&mut self, _client: &Arc<AppCtx>) {}

    fn on_scroll(&mut self, _client: &Arc<AppCtx>) {}

    fn on_hover(&mut self, _client: &Arc<AppCtx>, _mode: HoverMode) {}
}

impl Drop for TextBox {
    fn drop(&mut self) {
        let ctx = ctx();
        for glyph in self.texts.drain(..) {
            ctx.renderer.remove_glyph(glyph);
        }
    }
}