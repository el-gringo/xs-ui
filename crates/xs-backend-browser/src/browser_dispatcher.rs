use std::{
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use xs_core::{AppContext, BackendRequest, BackendResponse, Renderer, TextSystem};
use xs_layout::Image;

use crate::bindings;

pub struct BrowserDispatcher;
impl BrowserDispatcher {
    pub fn read_messages(
        &mut self,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        text_system: &mut Box<dyn TextSystem>,
        tx: Sender<BackendResponse>,
        rx: Rc<Receiver<BackendResponse>>,
    ) {
        while let Some(message) = app_context.messages.pop() {
            match message {
                BackendRequest::LoadImage(src) => {
                    self.load_image(src, tx.clone());
                }
                BackendRequest::Text(text, text_properties) => {
                    self.text(text_system, text, text_properties, tx.clone());
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
                    let texture = renderer.load_texture(&bytes, -1, width, height, true);
                    let image = Image::new(width, height, texture, bytes, true);
                    app_context.texts.insert(text, Some(image));
                }
            }
        }
    }

    pub fn load_image(&mut self, src: String, tx: Sender<BackendResponse>) {
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

    pub fn text(
        &mut self,
        text_system: &mut Box<dyn TextSystem>,
        text: String,
        text_properties: xs_core::TextProperties,
        tx: std::sync::mpsc::Sender<xs_core::BackendResponse>,
    ) {
        if let Ok((bytes, width, height)) = text_system.render_text(&text, &text_properties) {
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
