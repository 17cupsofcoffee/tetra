use crate::math::Vec2;
use crate::Context;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
/// A button on a mouse.
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

/// Returns true if the specified mouse button is currently down.
pub fn is_mouse_button_down(ctx: &Context, button: MouseButton) -> bool {
    ctx.input.current_mouse_state.contains(&button)
}

/// Returns true if the specified mouse button is currently up.
pub fn is_mouse_button_up(ctx: &Context, button: MouseButton) -> bool {
    !ctx.input.current_mouse_state.contains(&button)
}

/// Returns true if the specified mouse button was pressed this tick.
pub fn is_mouse_button_pressed(ctx: &Context, button: MouseButton) -> bool {
    !ctx.input.previous_mouse_state.contains(&button)
        && ctx.input.current_mouse_state.contains(&button)
}

/// Returns true if the specified mouse button was released this tick.
pub fn is_mouse_button_released(ctx: &Context, button: MouseButton) -> bool {
    ctx.input.previous_mouse_state.contains(&button)
        && !ctx.input.current_mouse_state.contains(&button)
}

/// Get the X co-ordinate of the mouse.
pub fn get_mouse_x(ctx: &Context) -> f32 {
    ctx.input.mouse_position.x
}

/// Get the Y co-ordinate of the mouse.
pub fn get_mouse_y(ctx: &Context) -> f32 {
    ctx.input.mouse_position.y
}

/// Get the position of the mouse.
pub fn get_mouse_position(ctx: &Context) -> Vec2<f32> {
    Vec2::new(get_mouse_x(ctx), get_mouse_y(ctx))
}

pub(crate) fn set_mouse_button_down(ctx: &mut Context, btn: MouseButton) {
    ctx.input.current_mouse_state.insert(btn);
}

pub(crate) fn set_mouse_button_up(ctx: &mut Context, btn: MouseButton) {
    ctx.input.current_mouse_state.remove(&btn);
}

pub(crate) fn set_mouse_position(ctx: &mut Context, position: Vec2<f32>) {
    ctx.input.mouse_position = position;
}
