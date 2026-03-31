use std::{
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use xs_core::{AppContext, BackendRequest, BackendResponse, Renderer};
use xs_http::HttpClient;
use xs_layout::Image;
use zune_image::codecs::qoi::zune_core::{colorspace::ColorSpace, options::DecoderOptions};

pub struct WaylandDispatcher;

impl WaylandDispatcher {
    pub fn read_messages(
        &mut self,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        tx: Sender<BackendResponse>,
        rx: Rc<Receiver<BackendResponse>>,
    ) {
        app_context.dispatch();
        while let Some(message) = app_context.messages.pop() {
            match message {
                BackendRequest::LoadImage(src) => {
                    self.load_image(src, tx.clone());
                }
            }
        }
        if let Ok(message) = rx.try_recv() {
            match message {
                BackendResponse::ImageLoaded(src, image_handle, bytes, width, height) => {
                    let texture = renderer.load_texture(&bytes, image_handle, width, height, false);
                    let image = Image::new(width, height, texture, bytes, false);
                    app_context.textures.insert(src.clone(), Some(image));
                }
            }
        }
    }

    pub fn load_image(&mut self, src: String, tx: Sender<BackendResponse>) {
        std::thread::spawn(move || {
            let http = HttpClient::new();
            http.get(
                src.clone(),
                move |response| match zune_image::image::Image::read(
                    std::io::Cursor::new(response.body),
                    DecoderOptions::default(),
                ) {
                    Ok(mut image) => {
                        let (width, height) = image.dimensions();
                        let width = width as u32;
                        let height = height as u32;
                        let _ = image.convert_color(ColorSpace::RGBA);
                        let bytes = image.flatten_to_u8().first().unwrap().clone();
                        let _ = tx.send(BackendResponse::ImageLoaded(
                            src.clone(),
                            -1,
                            bytes,
                            width,
                            height,
                        ));
                    }
                    Err(error) => {
                        eprintln!("Backend::load_image error: {:?}", error);
                    }
                },
            );
        });
    }
}
