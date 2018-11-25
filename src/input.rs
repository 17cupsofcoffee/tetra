use glm::Vec2;

use Context;
use Key;

pub(crate) struct InputContext {
    pub(crate) current_key_state: [bool; 322],
    pub(crate) previous_key_state: [bool; 322],
    pub(crate) mouse_position: Vec2,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            current_key_state: [false; 322],
            previous_key_state: [false; 322],
            mouse_position: Vec2::zeros(),
        }
    }
}

pub fn is_key_down(ctx: &Context, key: Key) -> bool {
    ctx.input.current_key_state[key as usize]
}

pub fn is_key_up(ctx: &Context, key: Key) -> bool {
    !ctx.input.current_key_state[key as usize]
}

pub fn is_key_pressed(ctx: &Context, key: Key) -> bool {
    let i = key as usize;
    !ctx.input.previous_key_state[i] && ctx.input.current_key_state[i]
}

pub fn is_key_released(ctx: &Context, key: Key) -> bool {
    let i = key as usize;
    ctx.input.previous_key_state[i] && !ctx.input.current_key_state[i]
}

pub fn get_mouse_position(ctx: &Context) -> Vec2 {
    ctx.input.mouse_position
}

pub fn get_mouse_x(ctx: &Context) -> f32 {
    ctx.input.mouse_position.x
}

pub fn get_mouse_y(ctx: &Context) -> f32 {
    ctx.input.mouse_position.y
}