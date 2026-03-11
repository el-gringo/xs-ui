use std::{any::TypeId, rc::Rc};

use xs::{Render, div};

type CellRenderer<T> = dyn Fn(usize, &T) -> Rc<dyn Render>;

#[derive(Default)]
pub struct Grid<T> {
    pub items: Vec<T>,
    pub columns: u32,
    pub cell_height: f32,
    pub cell_width: f32,
    pub width: f32,
    pub height: f32,
    pub gap: f32,
    pub grow: bool,
    pub cell_renderer: Option<Box<CellRenderer<T>>>,
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

    pub fn grow(mut self, grow: bool) -> Self {
        self.grow = grow;
        self
    }

    pub fn cell_renderer(
        mut self,
        cell_renderer: impl Fn(usize, &T) -> Rc<dyn Render> + 'static,
    ) -> Self {
        self.cell_renderer = Some(Box::new(cell_renderer));
        self
    }

    fn calculate_width(&self) -> f32 {
        if self.width > 0. {
            self.width
        } else if self.columns > 0 && self.cell_width > 0. {
            self.columns as f32 * self.cell_width + (self.columns as f32 - 1.) * self.gap
        } else {
            self.width
        }
    }
}

impl<T: Default + 'static> Render for Grid<T> {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, _app: &mut xs::Context) -> Box<dyn xs::Element> {
        let cell_renderer = &self.cell_renderer;
        let calculated_width = self.calculate_width();

        let cells: Vec<Rc<dyn Render>> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if self.grow {
                    div()
                        .height(self.cell_height)
                        .grow(1.)
                        .child((cell_renderer.as_ref().unwrap())(index, item))
                        .into()
                } else {
                    div()
                        .width(self.cell_width)
                        .height(self.cell_height)
                        .child((cell_renderer.as_ref().unwrap())(index, item))
                        .into()
                }
            })
            .collect();

        let mut grid_div = div()
            .width(calculated_width)
            .height(self.height)
            .row()
            .wrap()
            .gap(self.gap)
            .children(cells);

        if self.grow {
            grid_div = grid_div.grow(1.);
        }

        grid_div.into()
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
