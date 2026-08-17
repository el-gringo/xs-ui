use xs_animation::Animate;
use xs_core::Context;

pub fn animate<T: Animate + 'static>(ctx: &mut Context, template: T, target: f32) -> f32 {
    let anim = ctx.use_state::<T>(move |_, _| template);
    let prev_time = ctx.use_state::<u128>(|_, _| 0);

    let current = anim.read().value();
    if current == 1.0 {
        return target;
    }

    let now = ctx.app_context.now;
    let dt = now - *prev_time.read();
    if dt > 0 && dt < 1000 {
        anim.mutate(|s| {
            s.tick(dt as f32 / 1000.0);
        });
    }
    prev_time.update(|| now);

    anim.read().value()
}
