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
    ctx.input.mouse_buttons_down.contains(&button)
}

/// Returns true if the specified mouse button is currently up.
pub fn is_mouse_button_up(ctx: &Context, button: MouseButton) -> bool {
    !ctx.input.mouse_buttons_down.contains(&button)
}

/// Returns true if the specified mouse button was pressed since the last update.
pub fn is_mouse_button_pressed(ctx: &Context, button: MouseButton) -> bool {
    ctx.input.mouse_buttons_pressed.contains(&button)
}

/// Returns true if the specified mouse button was released since the last update.
pub fn is_mouse_button_released(ctx: &Context, button: MouseButton) -> bool {
    ctx.input.mouse_buttons_released.contains(&button)
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

pub(crate) fn set_mouse_button_down(ctx: &mut Context, btn: MouseButton) -> bool {
    let was_up = ctx.input.mouse_buttons_down.insert(btn);

    if was_up {
        ctx.input.mouse_buttons_pressed.insert(btn);
    }

    was_up
}

pub(crate) fn set_mouse_button_up(ctx: &mut Context, btn: MouseButton) -> bool {
    let was_down = ctx.input.mouse_buttons_down.remove(&btn);

    if was_down {
        ctx.input.mouse_buttons_released.insert(btn);
    }

    was_down
}

pub(crate) fn set_mouse_position(ctx: &mut Context, position: Vec2<f32>) {
    ctx.input.mouse_position = position;
}
