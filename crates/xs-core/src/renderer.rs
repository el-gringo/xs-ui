use std::{collections::VecDeque, rc::Rc};

use xs_layout::{BoxConstraints, Color, ImageFormat, TextProperties, View};
use xs_math::Vector2;

use crate::{
    AppContext, AtlasInfo, Context, Element, Render, core::NodeId, navigation::NavigationNodeId,
};

type RcWidget = Rc<dyn Render>;

#[derive(Clone)]
struct StackItem {
    widget: RcWidget,
    depth: usize,
    index: usize,
    parent_index: Option<usize>,
    nav_id: NavigationNodeId,
    text_properties: TextProperties,
    color: Color,
    constraints: BoxConstraints,
}

pub trait Renderer {
    fn init(&mut self);
    fn draw_frame(&mut self, ctx: &AppContext, widget_tree: &View);
    fn load_font_atlas(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        atlas_info: &AtlasInfo,
    );
    fn load_texture(
        &mut self,
        bytes: &[u8],
        image_handler: i32,
        width: u32,
        height: u32,
        image_format: ImageFormat,
    ) -> u32;

    fn render(&mut self, app_context: &mut AppContext, widget: RcWidget) -> View {
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(
                app_context.screen.width as f32,
                app_context.screen.height as f32,
            ),
        };
        let mut stack: Vec<StackItem> = Default::default();
        stack.push(StackItem {
            widget: widget.clone(),
            depth: 0,
            index: 0,
            parent_index: None,
            nav_id: app_context.current_nav_node_id,
            text_properties: TextProperties::default(),
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            constraints: constraints.clone(),
        });

        // Store (parent_index, element) for each rendered element
        let mut rendered_elements: Vec<(Option<usize>, Box<dyn Element>, usize)> =
            Default::default();
        while let Some(item) = stack.pop() {
            let node_id: NodeId = (item.widget.tag(), item.depth, item.index);
            let mut ctx = Context::new(
                app_context,
                node_id,
                item.nav_id,
                item.text_properties,
                item.color,
                item.constraints.clone(),
            );
            let element = item.widget.render(&mut ctx);

            // Push children onto stack (in reverse order)
            for (child_index, child) in element.children().iter().enumerate() {
                let new_item = StackItem {
                    widget: child.clone(),
                    depth: item.depth + 1,
                    index: child_index,
                    parent_index: Some(rendered_elements.len()),
                    nav_id: ctx.nav_id,
                    text_properties: ctx.text_properties.clone(),
                    color: ctx.color,
                    constraints: ctx.constraints.clone(),
                };
                stack.push(new_item);
            }
            rendered_elements.push((item.parent_index, element, item.depth));
        }

        /*
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
                if let Some(parent_idx) = rendered_elements[j].0
                    && parent_idx == i
                    && let Some(child_view) = &views[j]
                {
                    children_views.push(child_view.clone());
                }
            }

            // Reverse to get correct order (first child first)
            children_views.reverse();

            let view = element.paint(app_context, &children_views);
            views[i] = Some(view);
        }
        */
        let mut current_depth = rendered_elements.last().unwrap().2;
        eprintln!("current_depth: {current_depth}");
        let mut child_views = Vec::new();
        for (_, element, depth) in rendered_elements.iter().rev() {
            eprintln!(
                "current_depth: {current_depth} depth: {depth} {}",
                element.id()
            );
            if *depth == current_depth {
                let view = element.paint(app_context, &[]);
                child_views.push(view);
            } else {
                let view = element.paint(app_context, &child_views);
                child_views.clear();
                child_views.push(view);
            }
            current_depth = *depth;
        }

        app_context.nav.update();
        // The root is the first element (index 0)
        child_views[0].clone()
    }
}

pub struct RendererDummy;
impl Renderer for RendererDummy {
    fn init(&mut self) {}

    fn draw_frame(&mut self, _ctx: &AppContext, _widget_tree: &View) {}

    fn load_font_atlas(
        &mut self,
        _font_family: &'static str,
        _font_weight: u16,
        _atlas_info: &AtlasInfo,
    ) {
    }

    fn load_texture(
        &mut self,
        _bytes: &[u8],
        _image_handler: i32,
        _width: u32,
        _height: u32,
        _image_format: ImageFormat,
    ) -> u32 {
        0
    }
}
