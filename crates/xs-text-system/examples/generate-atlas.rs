use std::{fs::File, io::Write};

use xs_text_system::{Atlas, AtlasAlpha, AtlasMsdf, AtlasSdf, FontRef};

pub fn main() {
    let binary_font = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    let filename = "atlas_alpha.pgm";
    let font = FontRef::try_from_slice(binary_font).unwrap();
    let atlas = AtlasAlpha::generate(&font);
    let atlas_image = &atlas.info().image;
    let width = atlas_image.width;
    let height = atlas_image.height;
    let mut file = File::create(filename).unwrap();
    let _ = file.write(format!("P5\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = file.write(&atlas_image.bytes) {
        println!("Generated: {filename} ({width}x{height}) {size}");
    }

    let filename = "atlas_sdf.ppm";
    let atlas = AtlasSdf::generate(&font);
    let atlas_image = &atlas.info().image;
    let width = atlas_image.width;
    let height = atlas_image.height;
    let mut file = File::create(filename).unwrap();
    let _ = file.write(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = file.write(&atlas_image.bytes) {
        println!("Generated: {filename} ({width}x{height}) {size}");
    }

    let filename = "atlas_msdf.ppm";
    let atlas = AtlasMsdf::generate(&font);
    let atlas_image = &atlas.info().image;
    let width = atlas_image.width;
    let height = atlas_image.height;
    let mut file = File::create(filename).unwrap();
    let _ = file.write(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = file.write(&atlas_image.bytes) {
        println!("Generated: {filename} ({width}x{height}) {size}");
    }
}
