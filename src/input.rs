use fnv::FnvHashSet;
use glm::Vec2;
pub use sdl2::keyboard::Keycode as Key;

use graphics;
use Context;

pub(crate) struct InputContext {
    pub(crate) current_key_state: FnvHashSet<Key>,
    pub(crate) previous_key_state: FnvHashSet<Key>,
    pub(crate) mouse_position: Vec2,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            current_key_state: FnvHashSet::default(),
            previous_key_state: FnvHashSet::default(),
            mouse_position: Vec2::zeros(),
        }
    }
}

pub fn is_key_down(ctx: &Context, key: Key) -> bool {
    ctx.input.current_key_state.contains(&key)
}

pub fn is_key_up(ctx: &Context, key: Key) -> bool {
    !ctx.input.current_key_state.contains(&key)
}

pub fn is_key_pressed(ctx: &Context, key: Key) -> bool {
    !ctx.input.previous_key_state.contains(&key) && ctx.input.current_key_state.contains(&key)
}

pub fn is_key_released(ctx: &Context, key: Key) -> bool {
    ctx.input.previous_key_state.contains(&key) && !ctx.input.current_key_state.contains(&key)
}

pub fn get_mouse_x(ctx: &Context) -> f32 {
    (ctx.input.mouse_position.x / graphics::get_window_width(ctx) as f32)
        * graphics::get_width(ctx) as f32
}

pub fn get_mouse_y(ctx: &Context) -> f32 {
    (ctx.input.mouse_position.y / graphics::get_window_height(ctx) as f32)
        * graphics::get_height(ctx) as f32
}

pub fn get_mouse_position(ctx: &Context) -> Vec2 {
    Vec2::new(get_mouse_x(ctx), get_mouse_y(ctx))
}
