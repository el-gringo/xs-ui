use std::collections::VecDeque;
use std::rc::Rc;

use xs_layout::{BoxConstraints, Color, ImageFormat, LayoutBox, LayoutTree, TextProperties, View};
use xs_math::Vector2;

use crate::{
    AppContext, AtlasInfo, Context, ContextValues, core::NodeId, element::AnyElement,
    navigation::NavigationNodeId,
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
    contexts: ContextValues,
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

    fn build_element_tree(
        &mut self,
        app_context: &mut AppContext,
        root: AnyElement,
        root_contexts: ContextValues,
    ) -> View {
        app_context.nav.update();
        // Layout runs in device pixels, because a style has already been scaled
        // into them by the time it gets here - so the root is constrained to the
        // drawing buffer, not to the logical size the app was declared at.
        let (width, height) = app_context.screen.framebuffer_size();
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(width as f32, height as f32),
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
            contexts: root_contexts,
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
            // The root render already saw the app's root scope and anything the
            // root element provided, such as a router's route context; each
            // descendant then starts from what its parent provided.
            ctx.contexts = item.contexts.clone();

            let mut element = item.element;
            while let Some(resolved) = element.inner.resolve(&mut ctx) {
                element = resolved;
            }

            element.inner.post_render(&mut ctx);

            for (child_index, child) in element.inner.children().iter().enumerate() {
                let new_item = StackItem {
                    element: child.clone(),
                    parent_index: Some(rendered_elements.len()),
                    child_index,
                    nav_id: ctx.nav_id,
                    text_properties: ctx.text_properties.clone(),
                    color: ctx.color,
                    constraints: ctx.constraints.clone(),
                    contexts: ctx.contexts.clone(),
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

        if app_context.nav.update() {
            // Focus is read while the tree is built, so a change made after
            // that - a route turning the page card into a list, say - is not in
            // this frame. Request another one rather than leaving the old focus
            // on screen until the next key.
            app_context.dirty = true;
        }
        view_map[0].clone().unwrap()
    }

    /// `ctx` is here for the mesh cache and nothing else: a node that builds
    /// geometry during layout has no other route to the app, and without it a
    /// cache could not outlive the frame that filled it.
    fn layout(
        &mut self,
        ctx: &AppContext,
        widget_tree: &View,
        width: f32,
        height: f32,
    ) -> LayoutTree {
        let mut tree = LayoutTree::with_meshes(Rc::clone(&ctx.meshes));
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

#[cfg(test)]
mod tests {
    use std::{any::TypeId, cell::RefCell, rc::Rc};

    use xs_layout::{DummyContainer, View};

    use crate::{
        AnyElement, AppContext, Context, ElementObject, Renderer, RendererDummy, TextSystemDummy,
    };

    #[derive(Clone)]
    struct Marker(&'static str);

    type Seen = Rc<RefCell<Vec<(&'static str, Option<&'static str>)>>>;

    /// An element that records the `Marker` it can see and optionally provides
    /// one of its own. It defers everything to `post_render`, which is where a
    /// real widget would read the context it needs.
    #[derive(Clone)]
    struct Probe {
        label: &'static str,
        provides: Option<Marker>,
        children: Vec<AnyElement>,
        seen: Seen,
    }

    impl Probe {
        fn new(label: &'static str, seen: &Seen) -> Self {
            Self {
                label,
                provides: None,
                children: Vec::new(),
                seen: Rc::clone(seen),
            }
        }

        fn provides(mut self, marker: Marker) -> Self {
            self.provides = Some(marker);
            self
        }

        fn child(mut self, child: Probe) -> Self {
            self.children.push(AnyElement::new(child));
            self
        }
    }

    impl ElementObject for Probe {
        fn tag(&self) -> TypeId {
            TypeId::of::<Marker>()
        }

        fn clone_box(&self) -> Box<dyn ElementObject> {
            Box::new(self.clone())
        }

        fn children(&self) -> &[AnyElement] {
            &self.children
        }

        fn post_render(&mut self, ctx: &mut Context) {
            let seen = ctx.use_context::<Marker>().map(|marker| marker.0);
            self.seen.borrow_mut().push((self.label, seen));
            if let Some(marker) = self.provides.clone() {
                ctx.provide(marker);
            }
        }

        fn paint(&self, _app_context: &mut AppContext, _views: &[View]) -> View {
            Rc::new(DummyContainer)
        }
    }

    #[test]
    fn a_provided_value_reaches_descendants_but_not_siblings() {
        let seen: Seen = Rc::default();
        let root = Probe::new("root", &seen)
            .provides(Marker("outer"))
            .child(
                Probe::new("provider", &seen)
                    .provides(Marker("inner"))
                    .child(Probe::new("grandchild", &seen)),
            )
            .child(Probe::new("sibling", &seen));

        let mut app_context = AppContext::new(Box::new(TextSystemDummy::default()));
        let root_contexts = app_context.contexts.clone();
        let mut renderer = RendererDummy;
        renderer.build_element_tree(&mut app_context, AnyElement::new(root), root_contexts);

        assert_eq!(
            *seen.borrow(),
            vec![
                ("root", None),
                ("provider", Some("outer")),
                // The sibling is built before the provider's child, and must
                // see the value the provider's parent gave, not the provider's.
                ("sibling", Some("outer")),
                ("grandchild", Some("inner")),
            ]
        );
    }
}
