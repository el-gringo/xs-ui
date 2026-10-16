use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use xs_layout::{Image, TextProperties};
use xs_platform_event::KeyEvent;

use crate::{
    BackendRequest, FocusManager, TextSystem, TextSystemDummy, TextVertices,
    core::NodeId,
    navigation::NavigationNodeId,
    state::{AnyState, StateId},
};

/// The app's drawing surface, in logical pixels plus a device pixel ratio.
///
/// `width` and `height` are what the app laid out against and what an app
/// reads back to size its root, so they stay in logical pixels on every
/// backend: css pixels on the web, surface coordinates under Wayland. `scale`
/// is how many device pixels one of those is worth, and
/// [`Screen::framebuffer_size`] is what GL and the compositor actually need.
///
/// Nothing downstream of a style is in logical pixels. A style is multiplied
/// by the scale once, before it reaches layout, so layout, the text atlas, the
/// clip rectangles and the GL viewport are all in the framebuffer's own units
/// and there is no second coordinate space to convert between. At `scale == 1.0`
/// - a TV panel, or any display at its native resolution - the two coincide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    /// Logical width, in css pixels.
    pub width: u32,
    /// Logical height, in css pixels.
    pub height: u32,
    /// Device pixels per logical pixel.
    pub scale: f32,
}

impl Screen {
    #[must_use]
    pub const fn new(width: u32, height: u32, scale: f32) -> Self {
        Self {
            width,
            height,
            scale,
        }
    }

    /// The drawing buffer size, in device pixels.
    ///
    /// This is `glViewport`'s size and the shaders' `u_resolution`, so the two
    /// must agree; both read it from here rather than from `width`/`height`.
    #[must_use]
    pub fn framebuffer_size(&self) -> (u32, u32) {
        (
            (self.width as f32 * self.scale).round().max(0.0) as u32,
            (self.height as f32 * self.scale).round().max(0.0) as u32,
        )
    }

    /// Sets the ratio, ignoring a reading that is not positive.
    ///
    /// A compositor or a browser that reports zero would otherwise collapse the
    /// viewport to nothing and the app would draw nothing at all, with no error
    /// to explain it.
    pub fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 && scale.is_finite() {
            self.scale = scale;
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            width: 0,
            height: 0,
            scale: 1.0,
        }
    }
}

pub struct AppContext {
    pub running: bool,
    pub now: u128,
    pub screen: Screen,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Vec<Box<dyn Any>>>,
    pub states: HashMap<NodeId, HashMap<StateId, AnyState>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
    pub text_system: Box<dyn TextSystem>,
    pub texts: HashMap<(String, TextProperties), Option<(TextVertices, u32)>>,
    pub textures: HashMap<String, Option<Image>>,
    pub requests: HashMap<(String, String), Option<String>>,
    pub messages: Vec<BackendRequest>,
    pub dirty: bool,
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext {
            //app,
            running: true,
            now: 0,
            key_down: None,
            screen: Screen::default(),
            nav: FocusManager::new(),
            current_node_id: (TypeId::of::<Self>(), 0, 0),
            current_nav_node_id: 0,
            nodes: Default::default(),
            states: Default::default(),
            text_system: Box::new(TextSystemDummy::default()),
            textures: Default::default(),
            texts: Default::default(),
            requests: Default::default(),
            messages: Default::default(),
            dirty: true,
        }
    }
}

impl AppContext {
    pub fn new(text_system: Box<dyn TextSystem>) -> Self {
        AppContext {
            text_system,
            ..Default::default()
        }
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
            self.dirty = true;
        }
    }

    pub fn load_image(&mut self, src: Option<String>) -> Option<Image> {
        if let Some(src) = src {
            self.textures
                .entry(src.clone())
                .or_insert_with(|| {
                    self.messages.push(BackendRequest::LoadImage(src));
                    None
                })
                .clone()
        } else {
            None
        }
    }

    pub fn add_texture(&mut self, src: String, image: Image) {
        self.textures.insert(src.clone(), Some(image));
        self.dirty = true;
    }

    pub fn reload(&mut self) {
        self.messages.push(BackendRequest::Reload);
    }
}

#[cfg(test)]
mod screen_tests {
    use super::Screen;

    #[test]
    fn a_display_at_its_native_resolution_gets_the_declared_size() {
        // Every TV panel, and any browser at 100% zoom, reports 1.0 - and that is
        // the case where the logical size and the drawing buffer coincide, so
        // nothing downstream has to be different.
        let screen = Screen::new(1920, 1080, 1.0);
        assert_eq!(screen.framebuffer_size(), (1920, 1080));
    }

    #[test]
    fn a_hi_dpi_display_gets_a_buffer_of_the_right_size() {
        // 1920x1080 of layout across a 2x panel is 3840x2160 of pixels to
        // rasterise into. Half of that would be an upscale, which is the exact
        // thing this is here to avoid.
        let screen = Screen::new(1920, 1080, 2.0);
        assert_eq!(screen.framebuffer_size(), (3840, 2160));
    }

    #[test]
    fn a_fractional_ratio_rounds_to_a_whole_buffer() {
        // A browser on a 1.5x display, or one zoomed to 125%.
        let screen = Screen::new(1920, 1080, 1.5);
        assert_eq!(screen.framebuffer_size(), (2880, 1620));
    }

    #[test]
    fn a_zero_or_nonsense_ratio_is_refused() {
        // A backend that reports zero would otherwise collapse the viewport to
        // nothing and the app would draw nothing, with nothing to explain it.
        let mut screen = Screen::new(1920, 1080, 2.0);
        for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            screen.set_scale(bad);
            assert_eq!(screen.scale, 2.0, "accepted {bad}");
            assert_eq!(screen.framebuffer_size(), (3840, 2160));
        }
    }

    #[test]
    fn a_surface_that_has_not_been_configured_yet_has_no_size() {
        let screen = Screen::default();
        assert_eq!(screen.scale, 1.0);
        assert_eq!(screen.framebuffer_size(), (0, 0));
    }
}
