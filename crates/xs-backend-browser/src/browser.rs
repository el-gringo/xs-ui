use std::{cell::RefCell, ffi::{CStr, c_char}, rc::Rc};

use crate::{
    bindings::{self, console_log},
    browser_event::{translate_keycode, translate_mod},
};
use xs_backend_core::{App, AppContext, Backend, Render, RootElement};
use xs_layout::Image;
use xs_platform_event::{Event, KeyEvent};
use xs_renderer_core::Renderer;

fn into_ptr(s: &str) -> (u32, u32) {
    let bytes = s.as_bytes();
    (bytes.as_ptr() as u32, bytes.len() as u32)
}

pub fn log(message: &str) {
    unsafe {
        let (ptr, len) = into_ptr(message);
        console_log(ptr, len);
    }
}

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
            Event::ImageLoaded(src, ptr) => {
                self.app_context.ressources.insert(src, Image(ptr.try_into().unwrap()));
                log(&format!("Event::ImageLoaded: {} {}", ptr, self.app_context.ressources.len()));
            },
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
        let app_context = AppContext::new();
        let root = Rc::new(Box::new(RootElement {}) as Box<dyn Render>);
        let inner = Rc::new(RefCell::new(BackendBrowserInner {
            renderer,
            app_context,
            root,
        }));
        GLOBAL.with(|global| *global.borrow_mut() = Some(inner.clone()));
        unsafe {
            bindings::init_canvas();
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
            let (ptr, len) = into_ptr(src);
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

#[unsafe(no_mangle)]
pub extern "C" fn on_image_loaded(src: &mut c_char, image_ptr: u32) {
    unsafe {
        let string = CStr::from_ptr(src).to_str().unwrap();
        let event = Event::ImageLoaded(string, image_ptr);
        tl_event_handler(event);
    }
}
