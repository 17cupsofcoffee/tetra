use sdl2::keyboard::Keycode;

use Context;

pub fn is_key_down(ctx: &Context, keycode: Keycode) -> bool {
    ctx.current_key_state[keycode as usize]
}

pub fn is_key_up(ctx: &Context, keycode: Keycode) -> bool {
    !ctx.current_key_state[keycode as usize]
}

pub fn is_key_pressed(ctx: &Context, keycode: Keycode) -> bool {
    let i = keycode as usize;
    ctx.current_key_state[i] && (ctx.current_key_state[i] != ctx.previous_key_state[i])
}

pub fn is_key_released(ctx: &Context, keycode: Keycode) -> bool {
    let i = keycode as usize;
    !ctx.current_key_state[i] && (ctx.current_key_state[i] != ctx.previous_key_state[i])
}
