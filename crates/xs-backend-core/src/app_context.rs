use std::time::Duration;

pub struct AppContext {
    pub running: bool,
    pub now: Duration,
    pub key_down: Option<u32>
}

impl AppContext {
    pub fn new(now: Duration) -> Self {
        AppContext {
            now,
            running: true,
            key_down: None
        }
    }

    pub fn on_key_down(&mut self) -> Option<u32>  {
        let key_down = self.key_down.clone();
        self.key_down = None;
        key_down
    }
}
