use std::{cell::RefCell, rc::Rc, sync::mpsc::channel};

use crate::{bindings, browser_dispatcher::BrowserDispatcher};

use xs_core::{AppContext, Application, Backend, Render, Renderer, TextSystem, set_app_context};
use xs_js_sys::{Closure, request_animation_frame};

pub struct BackendBrowser {
    elapsed: u128,
}

impl BackendBrowser {
    pub fn new() -> Self {
        Self { elapsed: 0 }
    }
}

impl Backend for BackendBrowser {
    fn run(
        &mut self,
        app: Application,
        app_context: AppContext,
        renderer: Box<dyn Renderer>,
        text_system: Box<dyn TextSystem>,
        root: Rc<dyn Render>
    ) {
        let app_context = Rc::new(RefCell::new(app_context));
        let renderer = Rc::new(RefCell::new(renderer));
        let text_system = Rc::new(RefCell::new(text_system));
        renderer.borrow_mut().init();

        set_app_context(&mut app_context.borrow_mut());
        bindings::init(app.width, app.height);
        let mut messgage_dispatcher = BrowserDispatcher;

        let (tx, rx) = channel();
        let rx = Rc::new(rx);
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move |_now| {
            let rx = rx.clone();
            let mut renderer = renderer.borrow_mut();
            let mut app_context = app_context.borrow_mut();
            let mut text_system = text_system.borrow_mut();
            messgage_dispatcher.read_messages(
                &mut app_context,
                &mut renderer,
                &mut text_system,
                tx.clone(),
                rx,
            );
            let widget_tree = renderer.render(&mut app_context, root.clone());
            renderer.draw_frame(&widget_tree);
            request_animation_frame(f.borrow_mut().as_mut().unwrap());
        }));
        request_animation_frame(g.borrow_mut().as_mut().unwrap());
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
    }
}
