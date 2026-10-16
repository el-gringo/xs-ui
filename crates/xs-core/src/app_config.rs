/// What the app declares about itself before a backend exists.
///
/// `width` and `height` are **logical** pixels - css pixels on the web, surface
/// coordinates under Wayland - and are the size the layout is built against.
/// The backend turns them into a drawing buffer by multiplying by the device
/// pixel ratio it discovers, which lives on [`Screen::scale`] rather than here:
/// 1920x1080 is a full HD panel at any ratio, and a HiDPI display is not a
/// different design.
pub struct AppConfig {
    pub id: &'static str,
    pub title: &'static str,
    pub width: i32,
    pub height: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            id: "app",
            title: "App",
            width: 1920,
            height: 1080,
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = id;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    pub fn height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }
}
