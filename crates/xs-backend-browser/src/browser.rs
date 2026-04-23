use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use crate::bindings;

use xs_core::{AppConfig, AppContext, Backend, BackendResponse, EventLoop, set_app_context};
use xs_js_sys::{Closure, request_animation_frame};

#[derive(Default)]
pub struct BackendBrowser {
    elapsed: u128,
}

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
        set_app_context(&mut app_context);
        let app_context = Rc::new(RefCell::new(app_context));
        bindings::initialize(app.width, app.height);
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move |_now| {
            let mut app_context = app_context.borrow_mut();
            event_loop.frame(&mut app_context);
            request_animation_frame(f.borrow_mut().as_mut().unwrap());
        }));
        request_animation_frame(g.borrow_mut().as_mut().unwrap());
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
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
