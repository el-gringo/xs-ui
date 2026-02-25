use crate::AppContext;

pub struct Application {
    pub id: &'static str,
    pub title: &'static str,
    pub width: i32,
    pub height: i32,
    pub ctx: AppContext,
}

impl Application {
    pub fn new() -> Self {
        Self {
            id: "app",
            title: "App",
            width: 1440,
            height: 810,
            ctx: AppContext::new(),
        }
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
