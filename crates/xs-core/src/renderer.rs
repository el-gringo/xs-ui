use std::rc::Rc;

use xs_layout::{TextProperties, View};

use crate::{AppContext, Context, Element, Render, core::NodeId, navigation::NavigationNodeId};

type RcWidget = Rc<Box<dyn Render>>;

#[derive(Clone)]
struct StackItem {
    widget: RcWidget,
    depth: usize,
    index: usize,
    parent_index: Option<usize>,
    nav_id: NavigationNodeId,
    text_properties: TextProperties
}

pub trait Renderer {
    fn init(&mut self);
    fn screen_width(&self) -> i32;
    fn screen_height(&self) -> i32;
    fn draw_frame(&mut self, widget_tree: &View);
    fn load_texture(&self, bytes: &[u8], width: u32, height: u32, alpha: bool) -> u32;
    fn render(&mut self, app_context: &mut AppContext, widget: RcWidget) -> View {
        let mut stack: Vec<StackItem> = vec![StackItem {
            widget: widget.clone(),
            depth: 0,
            index: 0,
            parent_index: None,
            nav_id: app_context.current_nav_node_id,
            text_properties: TextProperties::default()
        }];

        // Store (parent_index, element) for each rendered element
        let mut rendered_elements: Vec<(Option<usize>, Box<dyn Element>)> = Vec::new();
        while let Some(mut item) = stack.pop() {
            let node_id: NodeId = (item.widget.tag(), item.depth, item.index);
            let mut ctx = Context::new(app_context, node_id, item.nav_id, &mut item.text_properties);
            let element = item.widget.render(&mut ctx);

            // Get children before moving element
            let children: Vec<Rc<Box<dyn Render>>> = element.children().to_vec();

            let current_index = rendered_elements.len();
            rendered_elements.push((item.parent_index, element));

            // Push children onto stack (in reverse order)
            for (child_index, child) in children.iter().enumerate().rev() {

                let new_item = StackItem {
                    widget: Rc::clone(child),
                    depth: item.depth + 1,
                    index: child_index,
                    parent_index: Some(current_index),
                    nav_id: ctx.nav_id,
                    text_properties: ctx.text_properties.clone()
                };
                stack.push(new_item);
            }
        }

        // Now build views from bottom up
        // views[i] stores the rendered view for element at index i
        let mut views: Vec<Option<View>> = Vec::with_capacity(rendered_elements.len());

        for _ in 0..rendered_elements.len() {
            views.push(None);
        }

        // Process in reverse order (children before parents)
        for i in (0..rendered_elements.len()).rev() {
            let (_parent_index, element) = &rendered_elements[i];

            // Find all children of this element
            let mut children_views: Vec<View> = Vec::new();
            for j in (i + 1..rendered_elements.len()).rev() {
                if let Some(parent_idx) = rendered_elements[j].0 {
                    if parent_idx == i {
                        if let Some(child_view) = &views[j] {
                            children_views.push(child_view.clone());
                        }
                    }
                }
            }

            // Reverse to get correct order (first child first)
            children_views.reverse();

            let view = element.paint(app_context, &children_views);
            views[i] = Some(view);
        }

        app_context.nav.update();
        // The root is the first element (index 0)
        views[0].as_ref().unwrap().clone()
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

    fn draw_frame(&mut self, _widget_tree: &View) {}

    fn load_texture(&self, _bytes: &[u8], _width: u32, _height: u32, _alpha: bool) -> u32 {
        0
    }
}
