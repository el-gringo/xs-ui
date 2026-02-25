use std::{any::TypeId, rc::Rc};

use xs::{Render, div};

#[derive(Default)]
pub struct Grid<T> {
    pub items: Vec<T>,
    pub cell_height: f32,
    pub cell_width: f32,
    pub width: f32,
    pub height: f32
}

impl<T: Default + 'static> Grid<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        self
    }

    pub fn cell_width(mut self, cell_width: f32) -> Self {
        self.cell_width = cell_width;
        self
    }

    pub fn cell_height(mut self, cell_width: f32) -> Self {
        self.cell_width = cell_width;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<T: Default + 'static> Render for Grid<T> {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, _app: &mut xs::Application) -> Box<dyn xs::Element> {
        let cells: Vec<Rc<Box<dyn Render>>> = self.items.iter().map(|_item| {
            div()
                .width(self.cell_width)
                .height(self.cell_height)
                .into()
        }).collect();
        div()
            .width(self.width)
            .height(self.height)
            .children(cells)
            .into()
    }
}
