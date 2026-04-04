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
            width: 1440,
            height: 810,
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
