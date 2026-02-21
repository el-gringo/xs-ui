use std::{collections::HashMap, rc::Rc};

use xs_layout::{Layout, View};

use crate::{Application, Element, Render, core::NodeId};

type RcComponent = Rc<Box<dyn Render>>;
struct RenderedElement {
    element: Box<dyn Element>,
    //cx: Context<'a>,
    depth: usize,
}

pub trait Renderer {
    fn init(&mut self);
    fn screen_width(&self) -> i32;
    fn screen_height(&self) -> i32;
    fn draw_frame(&mut self, widget_tree: Box<dyn Layout>);
    fn render(&mut self, app: &mut Application, widget: Rc<Box<dyn Render>>) {
        let mut stack: Vec<(RcComponent, usize, usize)> = vec![(widget, 0, 0)];
        let mut rendered: Vec<RenderedElement> = Vec::new();

        while let Some((widget, depth, index)) = stack.pop() {
            let node_id: NodeId = (widget.tag(), depth, index);
            //let mut cx = Context::new(app, node_id);
            app.ctx.current_node_id = node_id;
            let element = widget.render(app);
            for (child_index, child) in element.children().iter().enumerate().rev() {
                stack.push((Rc::clone(child), depth + 1, child_index));
            }
            rendered.push(RenderedElement { element, depth });
        }

        rendered.sort_by_key(|b| std::cmp::Reverse(b.depth));
        if let Some(first) = rendered.first() {
            let current_depth = first.depth;
            let mut child_views = Vec::new();
            for RenderedElement {
                element,
                //mut cx,
                depth,
            } in rendered
            {
                if depth == current_depth {
                    let view = element.paint(app, &child_views);
                    child_views.push(view);
                } else {
                    let view = element.paint(app, &child_views);
                    child_views = vec![view];
                }
            }
        }
    }
}

pub struct RendererDummy;
impl Renderer for RendererDummy {
    fn init(&mut self) {}

    fn screen_width(&self) -> i32 {
        0
    }

    fn screen_height(&self) -> i32 {
        0
    }

    fn draw_frame(&mut self, _widget_tree: Box<dyn Layout>) {}
}
