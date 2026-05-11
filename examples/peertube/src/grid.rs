use std::{any::TypeId, rc::Rc};

use xs::{Color, Context, Element, NavigationNode, Render, div};

type CellRenderer<T> = dyn Fn(usize, &T) -> Rc<dyn Render>;

pub struct GridCell {
    child: Rc<dyn Render>,
    width: f32,
    height: f32,
    grow: bool,
    on_focus: Option<Box<dyn Fn() + 'static>>,
}

impl Render for GridCell {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let node = ctx.focus_handle(NavigationNode::new().id("grid-cell"));
        let mut cell = div().height(self.height).border(
            if node.has_focus() {
                Color::RED
            } else {
                Color::TRANSPARENT
            },
            4.,
        );
        if self.grow {
            cell = cell.grow(1., 0.);
        } else {
            cell = cell.width(self.width);
        }

        if let Some(callback) = &self.on_focus {
            let was_focused = ctx.use_state::<bool>(|_| false);
            let is_focused = node.has_focus();
            if is_focused && !*was_focused.read() {
                callback();
            }
            was_focused.update(|| is_focused);
        }

        cell.child(self.child.clone()).into()
    }

    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl From<GridCell> for Rc<dyn Render> {
    fn from(val: GridCell) -> Self {
        Rc::new(val)
    }
}

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
    pub grow: bool,
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

    fn render(&self, ctx: &mut xs::Context) -> Box<dyn xs::Element> {
        let offset = ctx.use_state::<i32>(|_| 0);
        let _nav = ctx.focus_handle(NavigationNode::new().grid(self.columns as usize).id("grid"));
        let cell_renderer = &self.cell_renderer;
        let calculated_width = self.calculate_width();
        let columns = self.columns.max(1) as usize;
        let gap = self.gap;
        let cell_width = self.cell_width;
        let cell_height = self.cell_height;
        let grow = self.grow;
        let rows_count = self.rows;

        let offset_value = std::cmp::max(*offset.read() - 1, 0);
        let offset_updater = ctx.updater::<i32>();
        let on_row_change = self.on_row_change.clone();

        let cells: Vec<Rc<dyn Render>> = self
            .items
            .iter()
            .enumerate()
            .skip(offset_value as _)
            .take((rows_count * columns as u32) as _)
            .map(move |(index, item)| {
                let on_focus = {
                    let updater = offset_updater.clone();
                    let cb = on_row_change.clone();
                    Some(Box::new(move || {
                        let row_index = ((index / columns) as f32).floor() as usize;
                        eprintln!("row_index: {row_index}");
                        if let Some(ref cb) = cb {
                            cb(row_index);
                        }
                        updater.update(|| row_index as i32);
                    }) as Box<dyn Fn() + 'static>)
                };
                GridCell {
                    child: (cell_renderer.as_ref().unwrap())(index, &item),
                    width: cell_width,
                    height: cell_height,
                    grow,
                    on_focus,
                }
                .into()
            })
            .collect();

        let mut grid_div = div()
            .width(calculated_width)
            .height(self.height)
            .row()
            .gap(gap)
            .wrap()
            .children(cells);

        if self.grow {
            grid_div = grid_div.grow(1., 0.);
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
