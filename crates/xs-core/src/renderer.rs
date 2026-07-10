use std::collections::VecDeque;

use xs_layout::{BoxConstraints, Color, ImageFormat, LayoutBox, LayoutTree, TextProperties, View};
use xs_math::Vector2;

use crate::{
    AppContext, AtlasInfo, Context, core::NodeId, element::AnyElement, navigation::NavigationNodeId,
};

#[derive(Clone)]
struct StackItem {
    element: AnyElement,
    parent_index: Option<usize>,
    child_index: usize,
    nav_id: NavigationNodeId,
    text_properties: TextProperties,
    color: Color,
    constraints: BoxConstraints,
}

pub trait Renderer {
    fn init(&mut self, ctx: &AppContext);

    /// Called on frames where the scene hasn't changed.
    /// Should issue a lightweight GPU command to keep the compositor alive.
    fn present(&mut self) {}

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

    fn build_element_tree(&mut self, app_context: &mut AppContext, root: AnyElement) -> View {
        app_context.nav.update();
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(
                app_context.screen.width as f32,
                app_context.screen.height as f32,
            ),
        };
        let mut stack: VecDeque<StackItem> = Default::default();
        stack.push_front(StackItem {
            element: root.clone(),
            parent_index: None,
            child_index: 0,
            nav_id: app_context.current_nav_node_id,
            text_properties: TextProperties::default(),
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            constraints: constraints.clone(),
        });

        let mut rendered_elements: Vec<(Option<usize>, AnyElement)> = Default::default();
        while let Some(item) = stack.pop_front() {
            let node_id: NodeId = (
                item.element.inner.tag(),
                item.parent_index.unwrap_or(0),
                item.child_index,
            );
            let mut ctx = Context::new(
                app_context,
                node_id,
                item.nav_id,
                item.text_properties,
                item.color,
                item.constraints.clone(),
            );

            let mut element = item.element;
            loop {
                match element.inner.resolve(&mut ctx) {
                    Some(resolved) => element = resolved,
                    None => break,
                }
            }

            element.inner.post_render(&mut ctx);

            let children: Vec<AnyElement> =
                element.inner.children().iter().cloned().collect();
            for (child_index, child) in children.into_iter().enumerate() {
                let new_item = StackItem {
                    element: child,
                    parent_index: Some(rendered_elements.len()),
                    child_index,
                    nav_id: ctx.nav_id,
                    text_properties: ctx.text_properties.clone(),
                    color: ctx.color,
                    constraints: ctx.constraints.clone(),
                };
                stack.push_back(new_item);
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
                .map(|&child_idx| view_map[child_idx].clone().unwrap())
                .collect();
            view_map[i] = Some(element.inner.paint(app_context, &child_views));
        }

        app_context.nav.update();
        view_map[0].clone().unwrap()
    }

    fn layout(&mut self, widget_tree: &View, width: f32, height: f32) -> LayoutTree {
        let mut tree = LayoutTree::new();
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(width, height),
        };

        let root_sbox = widget_tree.layout(&mut tree, &constraints);

        let root_lbox = LayoutBox::from_child(root_sbox, (0.0, 0.0));
        let root_id = tree.insert(root_lbox.clone());
        tree.set_root(Some(root_id));
        tree
    }

    fn render(&mut self, ctx: &AppContext, layout_tree: &LayoutTree);
}

fn process_element(
    app_context: &mut AppContext,
    element: &AnyElement,
    parent_index: usize,
    counter: &mut usize,
    nav_id: NavigationNodeId,
    text_properties: TextProperties,
    color: Color,
    constraints: BoxConstraints,
) -> View {
    let my_index = *counter;
    *counter += 1;
    let node_id: NodeId = (element.inner.tag(), parent_index, my_index);
    let mut ctx = Context::new(
        app_context,
        node_id,
        nav_id,
        text_properties,
        color,
        constraints,
    );

    let mut current = element.clone();
    loop {
        match current.inner.resolve(&mut ctx) {
            Some(resolved) => current = resolved,
            None => break,
        }
    }

    current.inner.post_render(&mut ctx);

    let child_nav_id = ctx.nav_id;
    let child_text_properties = ctx.text_properties.clone();
    let child_color = ctx.color;
    let child_constraints = ctx.constraints.clone();
    drop(ctx);

    let child_views: Vec<View> = current
        .inner
        .children()
        .iter()
        .map(|child| {
            process_element(
                app_context,
                child,
                my_index,
                counter,
                child_nav_id,
                child_text_properties.clone(),
                child_color,
                child_constraints.clone(),
            )
        })
        .collect();

    current.inner.paint(app_context, &child_views)
}

pub struct RendererDummy;
impl Renderer for RendererDummy {
    fn init(&mut self, _ctx: &AppContext) {}

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

    fn render(&mut self, _ctx: &AppContext, _layout_tree: &LayoutTree) {}
}
