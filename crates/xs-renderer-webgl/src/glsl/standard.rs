use crate::webgl;
use xs_math::{Rectangle, Vector2, Vector3};
use xs_layout::{BorderSide, Borders, Material};

use crate::bindings::*;

const VERTEX_SHADER: &str = include_str!("standard.vert");
const FRAGMENT_SHADER: &str = include_str!("standard.frag");


fn attribute_name(prefix: String, index: u32, suffix: &str) -> String {
    return prefix + &index.to_string() + suffix;
}

pub struct StandardShader {
    program: GLuint,
    viewport: Vector2
}

impl StandardShader {
    pub fn new() -> StandardShader {
        let program = webgl::create_shader_program(VERTEX_SHADER, FRAGMENT_SHADER);
        StandardShader {
            program,
            viewport: Vector2::zero()
        }
    }

    pub fn set_viewport(&mut self, viewport: Vector2) {
        webgl::set_viewport(0, 0, viewport.x as i32, viewport.y as i32);
        self.viewport = viewport;
    }

    pub fn paint_rect(&mut self, rect: Rectangle, material: &Material, texture: Option<usize>) {
        let min_x = rect.x;
        let min_y = rect.y;
        let max_x = rect.x + rect.width;
        let max_y = rect.y + rect.height;
        webgl::set_uniform_vec2(self.program, "u_viewport", self.viewport.into());
        webgl::set_uniform_vec4(self.program, "u_color",    material.fill.into());
        webgl::set_uniform_vec2(self.program, "u_rect_min", [min_x, min_y]);
        webgl::set_uniform_vec2(self.program, "u_rect_max", [max_x, max_y]);
        self.set_borders(material.borders);
        if let Some(texture) = texture {
            webgl::set_uniform1i(self.program, "u_texture_enabled", 1);
            webgl::set_uniform1i(self.program, "u_texture", texture as i32);
        } else {
            webgl::set_uniform1i(self.program, "u_texture_enabled", 0);
        }
        let vertices: [Vector3; 6] = [
            Vector3::new(min_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(max_x, max_y, 0.0),
        ];
        webgl::draw_mesh(self.program, "a_position", vertices.to_vec());
    }

    fn set_borders(&mut self, borders: Borders) {
        self.set_border(0, borders.top);
        self.set_border(1, borders.bottom);
        self.set_border(2, borders.left);
        self.set_border(3, borders.right);
    }

    fn set_border(&mut self, i: usize, maybe_border: Option<BorderSide>) {
        let prefix = String::from("u_borders[");
        match maybe_border {
            Some(border) => {
                let BorderSide { color, width } = border;
                webgl::set_uniform_i32(self.program, &attribute_name(prefix.clone(), i as u32, "].enabled"), 1);
                webgl::set_uniform_f32(self.program, &attribute_name(prefix.clone(), i as u32, "].width"), width);
                webgl::set_uniform_vec4(self.program, &attribute_name(prefix.clone(), i as u32, "].color"), color.into());
            },
            None => {
                webgl::set_uniform_i32(self.program, &attribute_name(prefix.clone(), i as u32, "].enabled"), 0);
            }
        };
    }
}
