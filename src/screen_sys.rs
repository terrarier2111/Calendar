use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use winit::dpi::{PhysicalPosition, Position};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::render::{ctx, Model};
use crate::ui::Container;

pub type AppCtx = ();

pub trait Screen: Send + Sync {
    // Called once
    fn init(&mut self, _ctx: &Arc<AppCtx>) {}
    fn deinit(&mut self, _ctx: &Arc<AppCtx>) {}

    // May be called multiple times
    fn on_active(&mut self, _ctx: &Arc<AppCtx>);
    fn on_deactive(&mut self, _ctx: &Arc<AppCtx>);

    // Called every frame the screen is active
    fn tick(&mut self, _ctx: &Arc<AppCtx>);

    // Events
    fn on_scroll(&mut self, _x: f64, _y: f64) {}

    fn on_resize(&mut self, _ctx: &Arc<AppCtx>) {} // TODO: make non-optional!

    fn on_key_press(&mut self, screen_sys: &Arc<ScreenSystem>, key: PhysicalKey, down: bool) {
        if key == PhysicalKey::Code(KeyCode::Escape) && !down && self.is_closable() {
            screen_sys.pop_screen();
        }
    }

    fn on_char_receive(&mut self, _received: char) {}

    fn is_closable(&self) -> bool {
        false
    }

    fn is_tick_always(&self) -> bool {
        false
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn ty(&self) -> ScreenType {
        ScreenType::Other(String::new())
    }

    fn container(&self) -> &Arc<Container>;

    fn clone_screen(&self) -> Box<dyn Screen>;
}

impl Clone for Box<dyn Screen> {
    fn clone(&self) -> Box<dyn Screen> {
        self.clone_screen()
    }
}

#[derive(Eq, PartialEq)]
pub enum ScreenType {
    Other(String), // FIXME: maybe convert this into a "&'a str" or maybe even into a "&'static str"
    Chat,
    InGame, // FIXME: rework all the variations of this type!
}

#[derive(Clone)]
struct ScreenInfo {
    screen: Arc<Mutex<Box<dyn Screen>>>,
    active: bool,
    last_width: i32,
    last_height: i32,
}

#[derive(Default)]
pub struct ScreenSystem {
    screens: Arc<RwLock<Vec<ScreenInfo>>>,
    pre_computed_screens: Arc<RwLock<Vec<Box<dyn Screen>>>>,
    lowest_offset: AtomicIsize,
}

impl ScreenSystem {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_screen(&self, screen: Box<dyn Screen>) {
        let new_offset = self.pre_computed_screens.read().unwrap().len() as isize;
        self.pre_computed_screens
            .write()
            .unwrap()
            .push(screen);
        let _ = self.lowest_offset.compare_exchange(
            -1,
            new_offset,
            Ordering::Acquire,
            Ordering::Relaxed,
        );
    }

    pub fn close_closable_screens(&self) {
        while self.is_current_closable() {
            self.pop_screen();
        }
    }

    pub fn pop_screen(&self) {
        let mut pre_computed_screens = self.pre_computed_screens.write().unwrap();
        if pre_computed_screens.last().is_some() {
            pre_computed_screens.pop();
            let new_offset = pre_computed_screens.len() as isize;
            let _ = self.lowest_offset.fetch_update(
                Ordering::AcqRel,
                Ordering::Acquire,
                |curr_offset| {
                    if curr_offset == -1 || new_offset < curr_offset {
                        Some(new_offset)
                    } else {
                        None
                    }
                },
            );
        }
    }

    pub fn replace_screen(&self, screen: Box<dyn Screen>) {
        self.pop_screen();
        self.push_screen(screen);
    }

    pub fn is_current_closable(&self) -> bool {
        if let Some(last) = self.pre_computed_screens.read().unwrap().last() {
            return last.is_closable();
        }
        false
    }

    pub fn is_current_ingame(&self) -> bool {
        if let Some(last) = self.pre_computed_screens.read().unwrap().last() {
            return last.ty() == ScreenType::InGame;
        }
        false
    }

    pub fn is_any_ingame(&self) -> bool {
        for screen in self
            .pre_computed_screens
            .read()
            .unwrap()
            .iter()
            .rev()
        {
            if screen.ty() == ScreenType::InGame {
                return true;
            }
        }
        false
    }

    pub fn current_screen_ty(&self) -> ScreenType {
        if let Some(last) = self.pre_computed_screens.read().unwrap().last() {
            return last.ty();
        }
        ScreenType::Other(String::new())
    }

    pub fn receive_char(&self, received: char) {
        if let Some(screen) = self.screens.read().unwrap().last() {
            screen
                .screen
                .lock()
                .unwrap()
                .on_char_receive(received);
        }
    }

    pub fn press_key(self: &Arc<Self>, key: PhysicalKey, down: bool) {
        if let Some(screen) = self.screens.read().unwrap().last() {
            let mut screen = screen
            .screen
            .lock()
            .unwrap();
            if key == PhysicalKey::Code(KeyCode::Escape) && !down && screen.is_closable() {
                drop(screen);
                self.pop_screen();
                return;
            }
            screen.on_key_press(self, key, down);
        }
    }

    pub fn on_mouse_click(&self, ctx: &Arc<AppCtx>, pos: (f64, f64)) {
        if let Some(screen) = self.screens.read().unwrap().last() {
            screen
                .screen
                .lock()
                .unwrap()
                .container().on_mouse_click(ctx, pos);
        }
    }

    #[allow(unused_must_use)]
    pub fn tick(
        self: &Arc<Self>,
        app_ctx: &Arc<AppCtx>,
        window: &Window,
    ) -> Vec<Model> {
        println!("tick!");
        let ctx = ctx();
        let lowest = self.lowest_offset.load(Ordering::Acquire);
        if lowest != -1 {
            let screens_len = self.screens.read().unwrap().len();
            let was_closable = if screens_len > 0 {
                self.screens
                    .read()
                    .unwrap()
                    .last()
                    .as_ref()
                    .unwrap()
                    .screen
                    .lock()
                    .unwrap()
                    .is_closable()
            } else {
                false
            };
            if lowest <= screens_len as isize {
                for _ in 0..(screens_len as isize - lowest) {
                    let screen = self.screens.write().unwrap().pop().unwrap();
                    let active = screen.active;
                    let mut screen = screen.screen.lock().unwrap();

                    if active {
                        screen.on_deactive(app_ctx);
                    }
                    screen.deinit(app_ctx);
                }
            }
            for screen in self
                .pre_computed_screens
                .read()
                .unwrap()
                .iter()
                .skip(lowest as usize)
            {
                let mut screens = self.screens.write().unwrap();
                let idx = (screens.len() as isize - 1).max(0) as usize;
                screens.push(ScreenInfo {
                    screen: Arc::new(Mutex::new(screen.clone())),
                    active: false,
                    last_width: -1,
                    last_height: -1,
                });
                let last = screens.get_mut(idx);
                if let Some(last) = last {
                    if last.active {
                        last.active = false;
                        let mut screen = last.screen
                        .lock()
                        .unwrap();
                        screen.on_deactive(app_ctx);
                        if !screen.is_tick_always() {
                            ctx.renderer.clear_glyphs(); // FIXME: this can also do unwanted things as not all glyphs may belong to the current (non-ticking screen)
                        }
                    }
                }
                let current = screens.last_mut().unwrap();
                let curr_screen = current.screen.clone();
                let mut curr_screen = curr_screen.lock().unwrap();
                curr_screen.init(app_ctx);
                current.active = true;
                curr_screen.on_active(app_ctx);
            }
            self.lowest_offset.store(-1, Ordering::Release);
            if !was_closable {
                let (width, height) = ctx.renderer.dimensions.get();
                window.set_cursor_position(Position::Physical(PhysicalPosition::new(
                    (width / 2) as i32,
                    (height / 2) as i32,
                )));
            }
        }

        let len = self.screens.clone().read().unwrap().len();
        if len == 0 {
            return vec![];
        }
        // Update state for screens
        let tmp = self.screens.clone();
        let mut tmp = tmp.write().unwrap();
        let current = tmp.last_mut().unwrap();
        if !current.active {
            current.active = true;
            current
                .screen
                .lock()
                .unwrap()
                .on_active(app_ctx);
        }
        let (width, height) = ctx.renderer.dimensions.get();
        let last_transparent = current.screen.lock().unwrap().is_transparent();
        if current.last_width != width as i32 || current.last_height != height as i32 {
            if current.last_width != -1 && current.last_height != -1 {
                for screen in tmp.iter_mut().enumerate() {
                    let inner_screen = screen.1.screen.clone();
                    let mut inner_screen = inner_screen.lock().unwrap();
                    if inner_screen.is_tick_always() || screen.0 == len - 1 || (last_transparent && screen.0 == len - 2) {
                        inner_screen.on_resize(app_ctx);
                        drop(inner_screen);
                        let (width, height) = ctx.renderer.dimensions.get();
                        screen.1.last_width = width as i32;
                        screen.1.last_height = height as i32;
                    }
                }
            } else {
                let (width, height) = ctx.renderer.dimensions.get();
                current.last_width = width as i32;
                current.last_height = height as i32;
            }
        }
        let mut models = vec![];
        for screen in tmp.iter_mut().enumerate() {
            let inner_screen = screen.1.screen.clone();
            let mut inner_screen = inner_screen.lock().unwrap();
            if inner_screen.is_tick_always() || screen.0 == len - 1 {
                inner_screen.tick(app_ctx);
                let mut screen_models = inner_screen.container().build_models(app_ctx);
                models.append(&mut screen_models);
            }
        }
        models
    }

    pub fn on_scroll(&self, x: f64, y: f64) {
        if let Some(screen) = self.screens.clone().read().unwrap().last() {
            screen.screen.clone().lock().unwrap().on_scroll(x, y);
        }
    }
}