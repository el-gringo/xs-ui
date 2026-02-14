use std::{cell::RefCell, rc::Rc};

use crate::Backend;

pub struct App {
    pub id: &'static str,
    pub title: &'static str,
    pub width: i32,
    pub height: i32,
    pub backend: Option<Rc<RefCell<Box<dyn Backend>>>>
}
