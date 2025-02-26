use std::ffi::CString;

use xs_layout::{Color, Layout};

use crate::{egl_sys::{self, LibEgl}, gl_sys::{self, ClearColor}};


pub struct RendererOpenGl {

}

impl RendererOpenGl {
    pub fn new() -> Self {
        // Load EGL function pointers.
        let libegl = LibEgl::try_load().expect("can't load LibEGL");

        let mut major = 0;
        let mut minor = 0;

        unsafe {
            let egl_display = (libegl.eglGetDisplay.unwrap())(std::ptr::null_mut());
            //assert!(egl_display == , "can't get EGL platform display");

            assert!(
                (libegl.eglInitialize.unwrap())(egl_display, &mut major, &mut minor) != 0,
                "can't initialize EGL",
            );

            assert!(
                (libegl.eglBindAPI.unwrap())(egl_sys::EGL_OPENGL_ES_API) != 0,
                "can't bind EGL_OPENGL_ES_API",
            );

            // Choose framebuffer configuration.
            let cfg_attribs = [
                egl_sys::EGL_RED_SIZE,
                8,
                egl_sys::EGL_GREEN_SIZE,
                8,
                egl_sys::EGL_BLUE_SIZE,
                8,
                egl_sys::EGL_ALPHA_SIZE,
                8,
                // egl_sys::EGL_DEPTH_SIZE,
                // 24,
                // egl_sys::EGL_STENCIL_SIZE,
                // 8,
                egl_sys::EGL_RENDERABLE_TYPE,
                egl_sys::EGL_OPENGL_ES2_BIT,
                egl_sys::EGL_NONE
            ];

            let mut egl_config = 0 as egl_sys::EGLConfig;
            let mut matched_egl_configs = 0;
            assert!(
                (libegl.eglChooseConfig.unwrap())(
                    egl_display,
                    cfg_attribs.as_ptr() as _,
                    &mut egl_config,
                    1,
                    &mut matched_egl_configs
                ) != 0 && matched_egl_configs == 1,
                "eglChooseConfig failed",
            );

            // Create EGL context.
            let ctx_attribs = [
                egl_sys::EGL_CONTEXT_CLIENT_VERSION,
                2,
                egl_sys::EGL_NONE
            ];

            let egl_context = (libegl.eglCreateContext.unwrap())(
                egl_display,
                egl_config,
                egl_sys::EGL_NO_CONTEXT,
                ctx_attribs.as_ptr() as _,
            );
            assert!(!egl_context.is_null(), "eglCreateContext failed");

            // Load GL function pointers.
            gl_sys::load_with(|symbol| {
                let s = CString::new(symbol).unwrap();
                (libegl.eglGetProcAddress.unwrap())(s.as_ptr())
            });
        }
                    /*
                    // SAFETY: This creates an EGL surface. It's safe as long as we have valid EGL display, config, and window.
                    let surface = unsafe {(libegl.eglCreateWindowSurface.unwrap())(
                        egl_display,
                        egl_config,
                        window as _,
                        std::ptr::null_mut(),
                    )};

                    if unsafe {(libegl.eglMakeCurrent.unwrap())(egl_display, surface, surface, egl_context)} == 0 {
                        panic!();
                    }
                    */
        Self {}
    }

    pub fn render(&mut self, _widget_tree: Box<dyn Layout>) {
        let color = Color::black();
        unsafe {
            ClearColor(color.r, color.g, color.b, 1.0);
        }

        /*/
        let mut tree = LayoutTree::new();
        let (width, height) = WebGl::screen_size();
        let viewport = Vector2::new(width as f32, height as f32);
        self.shaders.set_viewport(viewport);

        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(width as f32, height as f32),
        };
        let root_sbox = widget_tree.layout(&mut tree, &constraints);
        let root_lbox = xs_layout::LayoutBox::from_child(root_sbox, (0.0, 0.0));
        let root_id = tree.insert(root_lbox);
        tree.set_root(Some(root_id));

        for (_, child, offset) in tree.iter() {
            let min = offset + Vector2::new(child.bounds.x, child.bounds.y) + child.margin.min();
            let max = Vector2::new(child.bounds.width, child.bounds.height) - child.margin.max();
            if let Some(material) = &child.material {
                let rect = Rectangle::new(min.x, min.y, max.x, max.y);
                //log(&format!("render {} {} {} {}", min.x, min.y, max.x, max.y));
                self.shaders.paint_rect(rect, &material);
            }
        }
        */
    }
}
