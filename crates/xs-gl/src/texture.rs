use xs_core::ImageFormat;
use xs_gl_sys::{
    GL_ALPHA, GL_CLAMP_TO_EDGE, GL_LINEAR, GL_LINEAR_MIPMAP_LINEAR, GL_RGBA, GL_TEXTURE_2D,
    GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T,
    GL_TEXTURE0, GL_UNPACK_ALIGNMENT, GL_UNSIGNED_BYTE, glActiveTexture, glBindTexture,
    glGenTextures, glGenerateMipmap, glPixelStorei, glTexImage2DByBytes, glTexImage2DByPointer,
    glTexParameteri,
};

fn is_power_of_2(value: i32) -> bool {
    (value & (value - 1)) == 0
}

pub fn load_texture(
    bytes: &[u8],
    image_handler: i32,
    width: u32,
    height: u32,
    image_format: ImageFormat,
) -> u32 {
    let internal_format = match image_format {
        ImageFormat::Alpha => GL_ALPHA,
        ImageFormat::Rgb => GL_RGBA,
        ImageFormat::Rgba => GL_RGBA,
    };
    let format = GL_RGBA;
    let textures = glGenTextures(1);
    let texture = textures[0];

    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, texture);
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S as _, GL_CLAMP_TO_EDGE as _);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T as _, GL_CLAMP_TO_EDGE as _);

    // Set filter modes
    if is_power_of_2(width as _) && is_power_of_2(height as _) {
        glTexParameteri(
            GL_TEXTURE_2D,
            GL_TEXTURE_MIN_FILTER,
            GL_LINEAR_MIPMAP_LINEAR as _,
        );
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as _);
        glGenerateMipmap(GL_TEXTURE_2D);
    } else {
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as _);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as _);
    }

    if image_handler > 0 {
        glTexImage2DByPointer(
            GL_TEXTURE_2D,
            0,
            internal_format as _,
            width as _,
            height as _,
            0,
            format,
            GL_UNSIGNED_BYTE,
            image_handler as _,
        );
    } else if !bytes.is_empty() {
        glTexImage2DByBytes(
            GL_TEXTURE_2D,
            0,
            internal_format as _,
            width as _,
            height as _,
            0,
            format,
            GL_UNSIGNED_BYTE,
            bytes,
        );
    }
    texture
}
