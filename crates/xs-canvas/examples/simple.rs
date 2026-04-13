use xs_canvas::Canvas;
use xs_core::EventLoop;
use xs_gl::{
    GL_COLOR_BUFFER_BIT, GL_FLOAT, GL_TRIANGLES, ShaderGlSl, glClear, glClearColor, glDrawArrays,
    glViewport,
};

const APP_WIDTH: i32 = 1440;
const APP_HEIGHT: i32 = 810;

#[derive(Default)]
struct SimpleCanvas {
    shader: Option<ShaderGlSl>,
}

impl SimpleCanvas {
    pub fn new() -> Self {
        Default::default()
    }
}

impl EventLoop for SimpleCanvas {
    fn initialize(&mut self, _ctx: &mut xs_core::AppContext) {
        self.shader = match ShaderGlSl::try_new(
            "attribute vec3 position;
			void main() { gl_Position = vec4( position, 1.0 ); }
            ",
            "precision lowp float;
            uniform float time;
            uniform vec2 resolution;

            void main() {
                vec2 uPos = ( gl_FragCoord.xy / resolution.xy );//normalize wrt y axis
               	uPos.x -= 0.5;
               	uPos.y -= 0.5;
               	vec3 color = vec3(0.0);
               	float vertColor = 0.0;
               	for( float i = 0.0; i < 10.0; ++i ) {
              		float t = time * (0.5);
              		uPos.y += sin( uPos.x*(i+1.0) + t+i/2.0 ) * 0.1;
              		float fTemp = abs(1.0 / uPos.y / 100.0);
              		vertColor += fTemp;
              		color += vec3( fTemp*(10.0-i)/10.0, fTemp*i/10.0, pow(fTemp,0.99)*1.5 );
               	}
               	vec4 color_final = vec4(color, 1.0);
               	gl_FragColor = color_final;
            }
            ",
            &["time", "resolution"],
            &[&[("position", 2, GL_FLOAT, false, 0, 0)]],
        ) {
            Ok(shader) => Some(shader),
            Err(error) => panic!("{}", error),
        }
    }

    fn frame(&mut self, ctx: &mut xs_core::AppContext) {
        let shader = self.shader.as_ref().unwrap();
        shader.use_program();
        shader.uniform_2f("resolution", APP_WIDTH as _, APP_HEIGHT as _);
        shader.uniform_1f("time", ctx.now as f32 / 1000.);

        #[rustfmt::skip]
        let vertices = &[
            -1.0, -1.0,
            -1.0,  1.0,
             1.0, -1.0,
            -1.0,  1.0,
             1.0, -1.0,
             1.0,  1.0
        ];
        shader.attribute_buffer_data("position", vertices);
        glDrawArrays(GL_TRIANGLES, 0, 6);
    }
}

pub fn main() {
    let app = SimpleCanvas::new();
    Canvas::new().run(app);
}
