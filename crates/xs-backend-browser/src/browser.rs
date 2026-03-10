use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use crate::bindings;

use xs_core::{
    AppContext, Application, Backend, BackendRequest, BackendResponse, Render, Renderer, TextSystem, set_app_context,
};
use xs_js_sys::{Closure, request_animation_frame};
use xs_layout::Image;

pub struct BackendBrowser {
    elapsed: u128,
    text_system: Box<dyn TextSystem>,
}

impl BackendBrowser {
    pub fn new(text_system: Box<dyn TextSystem>) -> Self {
        Self {
            elapsed: 0,
            text_system,
        }
    }
}

impl Backend for BackendBrowser {
    fn run(
        &mut self,
        app: Application,
        app_context: AppContext,
        mut renderer: Box<dyn Renderer>,
        root: Box<dyn Render>,
    ) {
        renderer.init();
        let root = Rc::new(root);
        let app_context = Rc::new(RefCell::new(app_context));
        let renderer = Rc::new(RefCell::new(renderer));

        bindings::init(app.width, app.height);

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move |_now| {
            let mut renderer = renderer.borrow_mut();
            let mut app_context = app_context.borrow_mut();
            set_app_context(&mut app_context);
            let widget_tree = renderer.render(&mut app_context, root.clone());
            renderer.draw_frame(&widget_tree);
            request_animation_frame(f.borrow_mut().as_mut().unwrap());
        }));
        request_animation_frame(g.borrow_mut().as_mut().unwrap());
    }

    fn read_messages(
        &mut self,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        tx: Sender<BackendResponse>,
        rx: &Receiver<BackendResponse>,
    ) {
        while let Some(message) = app_context.messages.pop() {
            match message {
                BackendRequest::LoadImage(src) => {
                    self.load_image(src, tx.clone());
                }
                BackendRequest::Text(text, text_properties) => {
                    self.text(text, text_properties, tx.clone());
                }
            }
        }
        if let Ok(message) = rx.try_recv() {
            match message {
                BackendResponse::ImageLoaded(src, image_handle, bytes, width, height) => {
                    let texture = renderer.load_texture(&bytes, image_handle, width, height, false);
                    let image = Image::new(width, height, texture, bytes, false);
                    app_context.textures.insert(src, Some(image));
                }
                BackendResponse::TextRendered(text, _text_properties, bytes, width, height) => {
                    let texture = renderer.load_texture(&bytes, 0, width, height, true);
                    let image = Image::new(width, height, texture, bytes, true);
                    app_context.texts.insert(text, Some(image));
                }
            }
        }
    }

    fn load_image(&mut self, src: String, tx: Sender<BackendResponse>) {
        bindings::load_image(src.clone(), move |image_handle| {
            let bytes = vec![];
            let (width, height) = (0, 0);
            let _ = tx.send(BackendResponse::ImageLoaded(
                src.clone(),
                image_handle,
                bytes,
                width,
                height,
            ));
        });
    }

    fn new_image(&mut self, bytes: Vec<u8>, width: u32, height: u32) -> u32 {
        bindings::new_image(bytes, width, height)
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
    }

    fn text(
        &mut self,
        text: String,
        text_properties: xs_core::TextProperties,
        tx: std::sync::mpsc::Sender<xs_core::BackendResponse>,
    ) {
        if let Ok((bytes, width, height)) = self.text_system.render_text(&text, &text_properties) {
            let _ = tx.send(BackendResponse::TextRendered(
                text,
                text_properties,
                bytes,
                width,
                height,
            ));
        }
    }
}
