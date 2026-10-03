use xs_animation::Animate;
use xs_layout::{BoxConstraints, Color, TextProperties, TextVertices};

use crate::{
    AnimationState, AppContext, FocusHandle, NavigationNode,
    navigation::NavigationNodeId,
    state::{AnyState, State},
};

pub struct Context<'a> {
    pub app_context: &'a mut AppContext,
    pub node_id: crate::core::NodeId,
    pub nav_id: NavigationNodeId,
    pub text_properties: TextProperties,
    pub color: Color,
    pub constraints: BoxConstraints,
}

impl<'a> Context<'a> {
    pub fn new(
        app_context: &'a mut AppContext,
        node_id: crate::core::NodeId,
        nav_id: NavigationNodeId,
        text_properties: TextProperties,
        color: Color,
        constraints: BoxConstraints,
    ) -> Self {
        Self {
            app_context,
            node_id,
            nav_id,
            text_properties,
            color,
            constraints,
        }
    }

    #[track_caller]
    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut Self) -> S) -> State<S>
    where
        S: 'static,
    {
        let caller = core::panic::Location::caller();
        let key = &format!("{}:{}:{}", caller.file(), caller.line(), caller.column());
        self.use_keyed_state(key, init)
    }

    pub fn use_keyed_state<S>(&mut self, key: &str, init: impl FnOnce(&mut Self) -> S) -> State<S>
    where
        S: 'static,
    {
        let node_id = self.node_id;
        let needs_init = {
            let entry = self.app_context.states.entry(node_id).or_default();
            !entry.contains_key(key)
        };
        if needs_init {
            let value = init(self);
            self.app_context
                .states
                .entry(node_id)
                .or_default()
                .insert(key.into(), AnyState::new(State::new(value)));
        }
        self.app_context
            .states
            .get_mut(&node_id)
            .expect("use_state: no state map for node_id")
            .get_mut(key)
            .unwrap_or_else(|| panic!("use_state: no state for state_id={key}"))
            .downcast::<State<S>>()
            .unwrap_or_else(|| panic!("use_state: downcast failed for state_id={key}"))
            .clone()
    }

    fn new_focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.app_context.nav.create_handle(self.nav_id, node);
        self.nav_id = handle.id;
        handle
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle_state = self.use_state::<FocusHandle>(|ctx| ctx.new_focus_handle(node));
        let result = handle_state.read().clone();
        self.nav_id = result.id;
        result
    }

    pub fn focus_handle_with_key(&mut self, key: &str, node: NavigationNode) -> FocusHandle {
        let state = self.use_keyed_state::<FocusHandle>(key, |ctx| ctx.new_focus_handle(node));
        let result = state.read().clone();
        self.nav_id = result.id;
        result
    }

    pub fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        max_width: f32,
        max_height: f32,
    ) -> TextVertices {
        self.app_context
            .text_system
            .render_text(text, text_properties, max_width, max_height)
    }

    pub fn animate<T: Animate + 'static>(&mut self, template: T, target: f32) -> f32 {
        let caller = core::panic::Location::caller();
        let key = &format!("{}:{}:{}", caller.file(), caller.line(), caller.column());

        let now = self.app_context.now;
        let anim = self.use_keyed_state::<AnimationState>(&format!("{key}-anim"), |_| {
            AnimationState::new(template, now)
        });
        let from = self.use_keyed_state::<f32>(&format!("{key}-from"), |_| target);
        let committed = self.use_keyed_state::<f32>(&format!("{key}-target"), |_| target);

        let committed_target = committed.get();
        if committed_target != target {
            let prev_from = from.get();
            let prev_progress = anim.read().value();
            let prev_result = prev_from + prev_progress * (committed_target - prev_from);
            committed.set(target);
            from.set(prev_result);
            anim.update(|s| s.restart(now));
        }

        let from_value = from.get();
        let target_value = committed.get();
        if from_value == target_value {
            return target_value;
        }

        if anim.read().is_finished() {
            from.set(target_value);
            return target_value;
        }

        let progress = {
            let mut progress = 0.0;
            anim.update(|s| progress = s.tick(now));
            progress
        };
        self.app_context.dirty = true;
        from_value + progress * (target_value - from_value)
    }
}

#[cfg(test)]
mod tests {
    use xs_animation::Spring;

    use crate::{AppContext, BoxConstraints, Color, Context, TextProperties, TextSystemDummy};

    fn ctx() -> Context<'static> {
        let app_context = Box::leak(Box::new(AppContext::new(Box::new(
            TextSystemDummy::default(),
        ))));
        Context::new(
            app_context,
            (std::any::TypeId::of::<u8>(), 0, 0),
            0,
            TextProperties::default(),
            Color::rgba(0., 0., 0., 1.),
            BoxConstraints::from_max((1000., 1000.)),
        )
    }

    #[test]
    fn animate_starts_at_from_and_springs_to_target() {
        let mut ctx = ctx();
        for frame in 0..300 {
            ctx.app_context.now = (frame + 1) * 16;
            let target = if frame < 2 { 0.0 } else { 320.0 };
            let value = ctx.animate(Spring::new(), target);
            if frame < 2 {
                assert_eq!(value, 0.0, "must hold still before target change");
                continue;
            }
            if frame == 2 {
                assert_eq!(value, 0.0, "must start from previous value");
            }
            assert!(
                value >= -1.0 && value <= 380.0,
                "value out of plausible range, got {value} at frame {frame}"
            );
        }
        ctx.app_context.now = 301 * 16;
        let final_value = ctx.animate(Spring::new(), 320.0);
        assert!(
            (final_value - 320.0).abs() < 1.0,
            "must reach target, got {final_value}"
        );
    }

    #[test]
    fn animate_marks_dirty_while_animating_and_idles_after() {
        let mut ctx = ctx();
        let mut dirty_frames = 0;
        for frame in 0..300 {
            ctx.app_context.now = (frame + 1) * 16;
            let target = if frame < 2 { 0.0 } else { 320.0 };
            ctx.animate(Spring::new(), target);
            if ctx.app_context.dirty {
                dirty_frames += 1;
                ctx.app_context.dirty = false;
            }
        }
        assert!(dirty_frames > 5, "should animate over several frames");
        assert!(
            dirty_frames < 120,
            "should settle, got {dirty_frames} dirty frames"
        );
        assert!(!ctx.app_context.dirty, "must be idle once settled");
    }
}
