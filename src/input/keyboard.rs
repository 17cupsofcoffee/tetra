use crate::Context;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
/// A key on a keyboard.
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    NumLock,
    NumPad1,
    NumPad2,
    NumPad3,
    NumPad4,
    NumPad5,
    NumPad6,
    NumPad7,
    NumPad8,
    NumPad9,
    NumPad0,
    NumPadPlus,
    NumPadMinus,
    NumPadMultiply,
    NumPadDivide,
    NumPadEnter,

    LeftCtrl,
    LeftShift,
    LeftAlt,
    RightCtrl,
    RightShift,
    RightAlt,

    Up,
    Down,
    Left,
    Right,

    Ampersand,
    Asterisk,
    At,
    Backquote,
    Backslash,
    Backspace,
    CapsLock,
    Caret,
    Colon,
    Comma,
    Delete,
    Dollar,
    DoubleQuote,
    End,
    Enter,
    Equals,
    Escape,
    Exclaim,
    GreaterThan,
    Hash,
    Home,
    Insert,
    LeftBracket,
    LeftParen,
    LessThan,
    Minus,
    PageDown,
    PageUp,
    Pause,
    Percent,
    Period,
    Plus,
    PrintScreen,
    Question,
    Quote,
    RightBracket,
    RightParen,
    ScrollLock,
    Semicolon,
    Slash,
    Space,
    Tab,
    Underscore,
}

/// Returns true if the specified key is currently down.
pub fn is_key_down(ctx: &Context, key: Key) -> bool {
    ctx.input.keys_down.contains(&key)
}

/// Returns true if the specified key is currently up.
pub fn is_key_up(ctx: &Context, key: Key) -> bool {
    !ctx.input.keys_down.contains(&key)
}

/// Returns true if the specified key was pressed since the last update.
pub fn is_key_pressed(ctx: &Context, key: Key) -> bool {
    ctx.input.keys_pressed.contains(&key)
}

/// Returns true if the specified key was released since the last update.
pub fn is_key_released(ctx: &Context, key: Key) -> bool {
    ctx.input.keys_released.contains(&key)
}

/// Returns an iterator of the keys that are currently down.
pub fn get_keys_down(ctx: &Context) -> impl Iterator<Item = &Key> {
    ctx.input.keys_down.iter()
}

/// Returns an iterator of the keys that were pressed since the last update.
pub fn get_keys_pressed(ctx: &Context) -> impl Iterator<Item = &Key> {
    ctx.input.keys_pressed.iter()
}

/// Returns an iterator of the keys that were released since the last update.
pub fn get_keys_released(ctx: &Context) -> impl Iterator<Item = &Key> {
    ctx.input.keys_released.iter()
}

pub(crate) fn set_key_down(ctx: &mut Context, key: Key) -> bool {
    let was_up = ctx.input.keys_down.insert(key);

    if was_up {
        ctx.input.keys_pressed.insert(key);
    }

    was_up
}

pub(crate) fn set_key_up(ctx: &mut Context, key: Key) -> bool {
    let was_down = ctx.input.keys_down.remove(&key);

    if was_down {
        ctx.input.keys_released.insert(key);
    }

    was_down
}
