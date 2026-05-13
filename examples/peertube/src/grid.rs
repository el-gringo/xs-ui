use std::{cmp::max, rc::Rc};

use xs::{NavigationNode, Render, div};

use crate::scroller::scroller;

type CellRenderer<T> = dyn Fn(usize, &T) -> Rc<dyn Render>;

#[derive(Default)]
pub struct Grid<T> {
    pub items: Vec<T>,
    pub columns: u32,
    pub rows: u32,
    pub cell_height: f32,
    pub cell_width: f32,
    pub width: f32,
    pub height: f32,
    pub gap: f32,
    pub cell_renderer: Option<Box<CellRenderer<T>>>,
    pub on_row_change: Option<Rc<dyn Fn(usize) + 'static>>,
}

impl<T: Default + 'static> Grid<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn items(mut self, items: Vec<T>) -> Self {
        self.items = items;
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

    pub fn columns(mut self, columns: u32) -> Self {
        self.columns = columns;
        self
    }

    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }

    pub fn on_row_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_row_change = Some(Rc::new(callback));
        self
    }

    pub fn cell_width(mut self, cell_width: f32) -> Self {
        self.cell_width = cell_width;
        self
    }

    pub fn cell_height(mut self, cell_height: f32) -> Self {
        self.cell_height = cell_height;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn cell_renderer(
        mut self,
        cell_renderer: impl Fn(usize, &T) -> Rc<dyn Render> + 'static,
    ) -> Self {
        self.cell_renderer = Some(Box::new(cell_renderer));
        self
    }
}

impl<T: Default + 'static> Render for Grid<T> {
    fn tag(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn render(&self, ctx: &mut xs::Context) -> Box<dyn xs::Element> {
        let focus_handle =
            ctx.focus_handle(NavigationNode::new().grid(self.columns as usize).id("grid"));
        let cell_renderer = &self.cell_renderer;
        let rows = self.rows;
        let columns = self.columns;
        let gap = self.gap;
        let cell_height = self.cell_height;

        let selected_index = focus_handle.selected_index();
        let selected_row = (selected_index as f32 / columns as f32).floor() as i32;
        let scroll_position = max(selected_row - 1, 0) * cell_height as i32;

        let first_row = max(selected_row - 2, 0) as i32;
        let skipped = (first_row * columns as i32) as i32;

        let cells: Vec<Rc<dyn Render>> = self
            .items
            .iter()
            .enumerate()
            .skip(skipped as _)
            .take((rows * columns) as _)
            .map(move |(index, item)| cell_renderer.as_ref().unwrap()(index, item))
            .collect();

        div()
            .width((self.cell_width + gap) * columns as f32)
            .child(
                scroller()
                    .scroll_to(scroll_position as _)
                    .children(vec![div().gap(gap).wrap().grow(1., 1.).children(cells)]),
            )
            .into()
    }
}

impl<T: Default + 'static> From<Grid<T>> for Rc<dyn Render> {
    fn from(val: Grid<T>) -> Self {
        Rc::new(val)
    }
}

pub fn grid<T>() -> Grid<T>
where
    T: Default + 'static,
{
    Grid::<T>::new()
}
