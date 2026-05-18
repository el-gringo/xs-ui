use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use crate::{bindings, browser_event};

use xs_core::{AppConfig, AppContext, Backend, BackendResponse, EventLoop};
use xs_js_sys::{Closure, EventEmitter, EventKeyDown, Node, document, request_animation_frame};
use xs_platform_event::KeyEvent;

#[derive(Default)]
pub struct BackendBrowser;

impl BackendBrowser {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Backend for BackendBrowser {
    fn run(
        &mut self,
        app: &AppConfig,
        mut app_context: AppContext,
        mut event_loop: Box<dyn EventLoop>,
    ) {
        event_loop.initialize(&mut app_context);
        app_context.screen.width = app.width as _;
        app_context.screen.height = app.height as _;
        let app_context = Rc::new(RefCell::new(app_context));
        bindings::initialize(app.width, app.height);

        let canvas = document().query_selector("canvas");
        let app_context_clone = app_context.clone();
        canvas.add_event_listener(
            "keydown",
            Closure::new(move |event: EventKeyDown| {
                let code = browser_event::_translate_keycode(event.key_code());
                let mods = browser_event::_translate_mod(event.modifiers());
                app_context_clone.borrow_mut().key_down = Some(KeyEvent::new(code, mods))
            }),
        );

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move |now| {
            let mut app_context = app_context.borrow_mut();
            app_context.now = now as _;
            event_loop.frame(&mut app_context);
            request_animation_frame(f.borrow_mut().as_mut().unwrap());
        }));
        request_animation_frame(g.borrow_mut().as_mut().unwrap());
    }

    #[cfg(feature = "image")]
    fn load_image(&mut self, src: String, tx: Sender<BackendResponse>) {
        load_image(src, tx);
    }
}

pub fn load_image(src: String, tx: Sender<BackendResponse>) {
    bindings::load_image(src.clone(), move |(handle, width, height)| {
        let bytes = vec![];
        let _ = tx.send(BackendResponse::ImageLoaded(
            src.clone(),
            handle as _,
            bytes,
            width,
            height,
        ));
    });
}
