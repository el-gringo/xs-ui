use std::rc::Rc;

use xs_layout::{BoxConstraints, Color, ImageFormat, TextProperties, View};
use xs_math::Vector2;

use crate::{
    AppContext, AtlasInfo, Context, Element, Render, core::NodeId, navigation::NavigationNodeId,
};

type RcWidget = Rc<dyn Render>;

#[derive(Clone)]
struct StackItem {
    widget: RcWidget,
    parent_index: Option<usize>,
    child_index: usize,
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
        app_context.nav.update();
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
            parent_index: None,
            child_index: 0,
            nav_id: app_context.current_nav_node_id,
            text_properties: TextProperties::default(),
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            constraints: constraints.clone(),
        });

        // Store (parent_index, element) for each rendered element
        let mut rendered_elements: Vec<(Option<usize>, Box<dyn Element>)> = Default::default();
        while let Some(item) = stack.pop() {
            let node_id: NodeId = (item.widget.tag(), item.parent_index.unwrap_or(0), item.child_index);
            let mut ctx = Context::new(
                app_context,
                node_id,
                item.nav_id,
                item.text_properties,
                item.color,
                item.constraints.clone(),
            );
            let element = item.widget.render(&mut ctx);
            element.post_render(&mut ctx);

            for (child_index, child) in element.children().iter().enumerate() {
                let new_item = StackItem {
                    widget: child.clone(),
                    parent_index: Some(rendered_elements.len()),
                    child_index,
                    nav_id: ctx.nav_id,
                    text_properties: ctx.text_properties.clone(),
                    color: ctx.color,
                    constraints: ctx.constraints.clone(),
                };
                stack.push(new_item);
            }
            rendered_elements.push((item.parent_index, element));
        }

        let elements_count = rendered_elements.len();
        let mut children_of: Vec<Vec<usize>> = vec![vec![]; elements_count];
        for (idx, (parent_idx, ..)) in rendered_elements.iter().enumerate() {
            if let Some(parent) = parent_idx {
                children_of[*parent].push(idx);
            }
        }

        let mut view_map: Vec<Option<View>> = vec![None; rendered_elements.len()];
        for i in (0..rendered_elements.len()).rev() {
            let (_, element) = &mut rendered_elements[i];
            let child_views: Vec<View> = children_of[i]
                .iter()
                .rev()
                .map(|&child_idx| view_map[child_idx].clone().unwrap())
                .collect();
            view_map[i] = Some(element.paint(app_context, &child_views));
        }

        app_context.nav.update();
        view_map[0].clone().unwrap()
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
