use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    bindings, browser_event::{translate_keycode, translate_mod}
};
use xs_backend_core::{AppContext, Backend, Render, RootElement};
use xs_js_sys::log;
use xs_layout::Image;
use xs_platform_event::{Event, KeyEvent};
use xs_renderer_core::Renderer;

struct BackendBrowserInner {
    renderer: Box<dyn Renderer>,
    app_context: AppContext,
    root: Rc<Box<dyn Render>>,
}

impl BackendBrowserInner {
    pub fn frame(&mut self) {
        self.app_context.dispatch();
        let frame = self.app_context.render(self.root.clone(), 0, 0);
        self.renderer.draw_frame(frame);
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::ImageLoaded(src_ptr, ptr) => {
                if let Some(image) = self.app_context.ressources.get_mut(&src_ptr) {
                    image.replace(Image(ptr.try_into().unwrap()));
                }
            }
            Event::RequestAnimationFrame => {
                self.frame();
            }
            Event::KeyDown(key_event) => {
                self.app_context.key_down = Some(key_event);
            }
        };
    }
}

pub struct BackendBrowser {
    id: &'static str,
    title: &'static str,
    inner: Rc<RefCell<BackendBrowserInner>>,
}

impl BackendBrowser {
    pub fn new(id: &'static str, title: &'static str, renderer: Box<dyn Renderer>) -> Self {
        let width = renderer.screen_width();
        let height = renderer.screen_height();
        let app_context = AppContext::new();
        let root = Rc::new(Box::new(RootElement {}) as Box<dyn Render>);
        let inner = Rc::new(RefCell::new(BackendBrowserInner {
            renderer,
            app_context,
            root,
        }));
        GLOBAL.with(|global| *global.borrow_mut() = Some(inner.clone()));
        unsafe {
            bindings::init_canvas(width, height);
        }
        Self { id, title, inner }
    }

    pub fn get_time(self) -> f64 {
        unsafe { bindings::now() }
    }

    pub fn log(message: &str) {
        log(message);
    }
}

impl Backend for BackendBrowser {
    fn run(&mut self, root: Box<dyn Render>) {
        log(&("runnning ".to_owned() + self.id + " " + self.title));
        self.inner.borrow_mut().root = Rc::new(root);
        unsafe {
            bindings::run();
        };
    }

    fn load_image(&mut self, src: &'static str) {
        unsafe {
            let (ptr, len) = {
                let bytes = src.as_bytes();
                (bytes.as_ptr() as u32, bytes.len() as u32)
            };
            bindings::load_image(ptr, len);
        }
    }
}

thread_local! {
    static GLOBAL: RefCell<Option<Rc<RefCell<BackendBrowserInner>>>> = RefCell::new(None);
}

fn tl_event_handler(event: Event) {
    GLOBAL.with(|global| {
        global
            .borrow_mut()
            .as_mut()
            .unwrap()
            .borrow_mut()
            .handle_event(event);
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn frame() {
    tl_event_handler(Event::RequestAnimationFrame);
}

#[unsafe(no_mangle)]
pub extern "C" fn key_down(key: u32, modifiers: u32, _repeat: bool) {
    let code = translate_keycode(key as _);
    let mods = translate_mod(modifiers as _);
    let event = KeyEvent::new(code, mods);
    tl_event_handler(Event::KeyDown(event));
}

// #[repr(C)]
// pub struct JsString {
//     data: *const u8,
//     len: usize,
// }

#[unsafe(no_mangle)]
pub extern "C" fn on_image_loaded(src_ptr: u32, image_ptr: u32) {
    let event = Event::ImageLoaded(src_ptr, image_ptr);
    tl_event_handler(event);
}
