use crate::math::Vec2;
use crate::Context;

/// A button on a mouse.
///
/// # Serde
///
/// Serialization and deserialization of this type (via [Serde](https://serde.rs/))
/// can be enabled via the `serde_support` feature.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[allow(missing_docs)]
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

/// Returns true if the user scrolled up since the last update.
pub fn is_mouse_scrolled_up(ctx: &Context) -> bool {
    get_mouse_wheel_movement(ctx).y > 0
}

/// Returns true if the user scrolled down since the last update.
pub fn is_mouse_scrolled_down(ctx: &Context) -> bool {
    get_mouse_wheel_movement(ctx).y < 0
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
    ctx.input.mouse_position
}

/// Get the amount that the mouse wheel moved since the last update.
///
/// Most 'normal' mice can only scroll vertically, but some devices can also scroll horizontally.
/// Use the Y component of the returned vector if you don't care about horizontal scroll.
///
/// Positive values correspond to scrolling up/right, negative values correspond to scrolling
/// down/left.
pub fn get_mouse_wheel_movement(ctx: &Context) -> Vec2<i32> {
    ctx.input.mouse_wheel_movement
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

pub(crate) fn apply_mouse_wheel_movement(ctx: &mut Context, wheel_movement: Vec2<i32>) {
    ctx.input.mouse_wheel_movement += wheel_movement;
}
