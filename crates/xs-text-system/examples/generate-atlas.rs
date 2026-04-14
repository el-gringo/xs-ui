use std::{fs::File, io::Write};

use xs_text_system::{Atlas, AtlasAlpha, AtlasMsdf, AtlasSdf};
use xs_text_system_glyph::FontRef;

pub fn main() {
    let font_size = 32.0;
    let binary_font = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
    let font = FontRef::try_from_slice(binary_font).unwrap();
    let (alpha_atlas, _format, alpha_data) = AtlasAlpha::generate(&font, font_size, 0.0);
    let filename = "atlas_alpha.pgm";
    let width = alpha_atlas.width();
    let height = alpha_atlas.height();
    let mut file = File::create(filename).unwrap();
    let _ = file.write(format!("P5\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = file.write(&alpha_data) {
        println!("Generated: {filename} ({width}x{height}) {size}");
    }

    let (sdf_atlas, _format, sdf_data) = AtlasSdf::generate(&font, font_size, 4.0);
    let filename = "atlas_sdf.ppm";
    let width = sdf_atlas.width();
    let height = sdf_atlas.height();
    let mut file = File::create(filename).unwrap();
    let _ = file.write(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = file.write(&sdf_data) {
        println!("Generated: {filename} ({width}x{height}) {size}");
    }

    let (msdf_atlas, _format, msdf_data) = AtlasMsdf::generate(&font, font_size, 4.0);
    let filename = "atlas_msdf.ppm";
    let width = msdf_atlas.width();
    let height = msdf_atlas.height();
    let mut file = File::create(filename).unwrap();
    let _ = file.write(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = file.write(&msdf_data) {
        println!("Generated: {filename} ({width}x{height}) {size}");
    }
}
