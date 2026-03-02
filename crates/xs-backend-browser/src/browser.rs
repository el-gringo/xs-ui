use std::{cell::RefCell, rc::Rc};

use crate::{
    bindings,
    browser_event::{translate_keycode, translate_mod},
};

use xs_core::{AppContext, Application, Backend, Render, Renderer};
use xs_js_sys::{Closure, log, request_animation_frame};
use xs_layout::Image;
use xs_platform_event::{Event, KeyEvent};

struct BackendBrowserInner {
    renderer: Box<dyn Renderer>,
    app_context: AppContext,
    root: Rc<Box<dyn Render>>
}

impl BackendBrowserInner {
    pub fn frame(&mut self) {}

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::ImageLoaded(src_ptr, ptr, width, height, texture_index) => {
                if let Some(image) = self.app_context.textures.get_mut(&src_ptr) {
                    log(&format!("Event::ImageLoaded: {texture_index}"));
                    let ptr: i32 = ptr.try_into().unwrap();
                    image.replace(Image::new(ptr, width, height, texture_index));
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
    elapsed: u128
};
impl BackendBrowser {
    pub fn new() -> Self {
        Self { elapsed: 0 }
    }

    pub fn get_time(self) -> f64 {
        unsafe { bindings::now() }
    }

    pub fn log(message: &str) {
        log(message);
    }


}

impl Backend for BackendBrowser {
    fn run(
        &mut self,
        app: &mut Application,
        renderer: &mut Box<dyn Renderer>,
        root: Box<dyn Render>,
    ) {
        //log(&("runnning ".to_owned() + self.app.id + " " + self.app.title));
        unsafe {
            bindings::init(app.width, app.height);
        };
        let root = Rc::new(root);
        let widget_tree = renderer.render(app, root.clone());
        renderer.draw_frame(&widget_tree);
        request_animation_frame(&Closure::new(|| {
            let widget_tree = renderer.render(app, root.clone());
            renderer.draw_frame(&widget_tree);
        }));
        //app.ctx.nav.update();


        // let f = Rc::new(RefCell::new(None));
        // let g = f.clone();
        // *g.borrow_mut() = Some(Closure::new(|| {
        //     log("request_animation_frame closure");
        //     app.with_context(|mut ctx| {
        //         let frame = ctx.render(app, root.clone(), 0, 0);
        //         ctx.renderer.draw_frame(frame);
        //         ctx.nav.update();
        //     })
        // }));
        // request_animation_frame(g.borrow().as_ref().unwrap());

    }
    fn load_image(&mut self, src: &'static str, texture_index: u32) {
        unsafe {
            let (ptr, len) = {
                let bytes = src.as_bytes();
                (bytes.as_ptr() as u32, bytes.len() as u32)
            };
            bindings::load_image(ptr, len, texture_index);
        }
    }

    fn new_image(&mut self, mut array: Vec<u8>, width: u32) -> u32 {
        unsafe {
            let ptr = array.as_mut_ptr() as u32;
            let len = array.len().try_into().unwrap();
            bindings::new_image(ptr, len, width)
        }
    }

    fn text(&mut self, text: &'static str) {
        todo!()
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
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
pub extern "C" fn on_image_loaded(
    src_ptr: u32,
    _str_len: u32,
    image_ptr: u32,
    width: u32,
    height: u32,
    texture_index: u32,
) {
    let event = Event::ImageLoaded(src_ptr, image_ptr, width, height, texture_index);
    tl_event_handler(event);
}
