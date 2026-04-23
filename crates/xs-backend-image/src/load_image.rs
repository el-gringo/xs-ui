use std::sync::mpsc::Sender;

use xs_core::BackendResponse;
use xs_http::HttpClient;

pub fn load_image(src: String, tx: Sender<BackendResponse>) {
    use zune_image::codecs::qoi::zune_core::{colorspace::ColorSpace, options::DecoderOptions};
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
