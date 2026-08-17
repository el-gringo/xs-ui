use std::cmp::max;
use std::rc::Rc;

use xs_core::{AnyElement, Context, IntoElement, NavigationNode, ParentElement, Render};

use crate::{div, scroller::scroller};

type CellRenderer<T> = dyn Fn(usize, &T, &mut Context) -> AnyElement;

#[derive(Default)]
pub struct Grid<T> {
    pub items: Rc<Vec<T>>,
    pub columns: u32,
    pub rows: u32,
    pub cell_height: f32,
    pub cell_width: f32,
    pub width: f32,
    pub height: f32,
    pub gap: f32,
    pub cell_renderer: Option<Box<CellRenderer<T>>>,
    pub on_row_change: Option<std::rc::Rc<dyn Fn(usize) + 'static>>,
}

impl<T: Default + 'static> Grid<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn items(mut self, items: Rc<Vec<T>>) -> Self {
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
        self.on_row_change = Some(std::rc::Rc::new(callback));
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
        cell_renderer: impl Fn(usize, &T, &mut Context) -> AnyElement + 'static,
    ) -> Self {
        self.cell_renderer = Some(Box::new(cell_renderer));
        self
    }
}

impl<T: Default + 'static> Render for Grid<T> {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let focus_handle =
            ctx.focus_handle(NavigationNode::new().grid(self.columns as usize).id("grid"));
        let cell_renderer = &self.cell_renderer;
        let columns = self.columns as usize;
        let gap = self.gap;
        let cell_height = self.cell_height;
        let window_rows = self.rows as usize;
        let buffer = 2usize;
        let render_rows = window_rows + buffer;
        let render_cells = render_rows * columns;

        let offset = ctx.use_state::<usize>(|_| 0);
        let old_offset = offset.get();

        let total_items = self.items.len();
        let total_rows = total_items.div_ceil(columns);

        let selected_index = focus_handle.selected_index();
        let selected_row = selected_index / columns;

        let mut new_offset = old_offset;
        if selected_row + 1 >= render_rows && old_offset + window_rows < total_rows {
            new_offset = old_offset + 1;
        }
        if selected_row == 0 && old_offset > 0 {
            new_offset = old_offset - 1;
        }

        new_offset = new_offset.min(total_rows.saturating_sub(render_rows));

        if new_offset != old_offset {
            offset.set(new_offset);

            let data_index = old_offset * columns + selected_index;
            let new_selected = data_index.saturating_sub(new_offset * columns);
            let max_valid = render_cells.saturating_sub(1);
            focus_handle.set_selected_index(new_selected.min(max_valid));

            if let Some(ref cb) = self.on_row_change {
                cb(new_offset);
            }
        }

        let data_start = new_offset * columns;
        let data_end = data_start + render_cells;

        let mut cell_rows: Vec<Vec<AnyElement>> = vec![Vec::with_capacity(columns); render_rows];
        for i in data_start..data_end {
            let row = (i - data_start) / columns;
            if i < total_items {
                cell_rows[row].push(cell_renderer.as_ref().unwrap()(i, &self.items[i], ctx));
            }
        }

        let adjusted_selected = focus_handle.selected_index();
        let adjusted_row = adjusted_selected / columns;
        let data_row = new_offset + adjusted_row;
        let scroll_position = max(data_row as i32 - 1, 0) * cell_height as i32;

        let rows: Vec<AnyElement> = cell_rows
            .into_iter()
            .enumerate()
            .map(|(row_index, cells)| {
                let data_row_index = new_offset + row_index;
                div()
                    .absolute()
                    .gap(gap)
                    .top(data_row_index as f32 * cell_height)
                    .height(cell_height)
                    .children(cells)
                    .into()
            })
            .collect();

        div().width((self.cell_width + gap) * columns as f32).child(
            scroller()
                .scroll_to(scroll_position as _)
                .children(vec![div().children(rows)]),
        )
    }
}

pub fn grid<T>() -> Grid<T>
where
    T: Default + 'static,
{
    Grid::<T>::new()
}
