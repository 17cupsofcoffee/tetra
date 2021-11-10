use std::fmt::{self, Display, Formatter};

use crate::Context;

/// A physical key on a keyboard.
///
/// This type represents keys based on their physical position, independent from the user's
/// active keyboard layout. The variants are named based on how the keys are labelled on a
/// US QWERTY keyboard. For example, `Key::A` is the key to the right of the Caps Lock,
/// even if the user is on an AZERTY keyboard.
///
/// This is used as the default representation as it allows non-QWERTY keyboard layouts
/// to be supported with minimal effort on the developer's part. However, you should
/// consider providing configurable input bindings too, for maximum accessibility.
///
/// If you need to determine what a key represents in the current keyboard layout (e.g.
/// to display button prompts, or for a config screen), you can use the [`get_key_label`]
/// function.
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

    Backquote,
    Backslash,
    Backspace,
    CapsLock,
    Comma,
    Delete,
    End,
    Enter,
    Equals,
    Escape,
    Home,
    Insert,
    LeftBracket,
    Minus,
    PageDown,
    PageUp,
    Pause,
    Period,
    PrintScreen,
    Quote,
    RightBracket,
    ScrollLock,
    Semicolon,
    Slash,
    Space,
    Tab,
}

/// A key, as represented by the current system keyboard layout.
///
/// This type represents keys based on how they are labelled and what character they generate.
/// For example, `KeyLabel::A` is the key to the right of the Caps Lock on a QWERTY keyboard,
/// whereas it is the key to the right of Tab on an AZERTY keyboard.
///
/// The main use case for `KeyLabel` is when you need to display a key name to the player
/// (e.g. in tutorials, or on an input binding screen). As such, it implements `Display`
/// in a UI-friendly way. You can get the label for a given key via [`get_key_label`],
/// and the key with a given label via [`get_key_with_label`].  
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
pub enum KeyLabel {
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

impl Display for KeyLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                KeyLabel::A => "A",
                KeyLabel::B => "B",
                KeyLabel::C => "C",
                KeyLabel::D => "D",
                KeyLabel::E => "E",
                KeyLabel::F => "F",
                KeyLabel::G => "G",
                KeyLabel::H => "H",
                KeyLabel::I => "I",
                KeyLabel::J => "J",
                KeyLabel::K => "K",
                KeyLabel::L => "L",
                KeyLabel::M => "M",
                KeyLabel::N => "N",
                KeyLabel::O => "O",
                KeyLabel::P => "P",
                KeyLabel::Q => "Q",
                KeyLabel::R => "R",
                KeyLabel::S => "S",
                KeyLabel::T => "T",
                KeyLabel::U => "U",
                KeyLabel::V => "V",
                KeyLabel::W => "W",
                KeyLabel::X => "X",
                KeyLabel::Y => "Y",
                KeyLabel::Z => "Z",
                KeyLabel::Num0 => "0",
                KeyLabel::Num1 => "1",
                KeyLabel::Num2 => "2",
                KeyLabel::Num3 => "3",
                KeyLabel::Num4 => "4",
                KeyLabel::Num5 => "5",
                KeyLabel::Num6 => "6",
                KeyLabel::Num7 => "7",
                KeyLabel::Num8 => "8",
                KeyLabel::Num9 => "9",
                KeyLabel::F1 => "F1",
                KeyLabel::F2 => "F2",
                KeyLabel::F3 => "F3",
                KeyLabel::F4 => "F4",
                KeyLabel::F5 => "F5",
                KeyLabel::F6 => "F6",
                KeyLabel::F7 => "F7",
                KeyLabel::F8 => "F8",
                KeyLabel::F9 => "F9",
                KeyLabel::F10 => "F10",
                KeyLabel::F11 => "F11",
                KeyLabel::F12 => "F12",
                KeyLabel::F13 => "F13",
                KeyLabel::F14 => "F14",
                KeyLabel::F15 => "F15",
                KeyLabel::F16 => "F16",
                KeyLabel::F17 => "F17",
                KeyLabel::F18 => "F18",
                KeyLabel::F19 => "F19",
                KeyLabel::F20 => "F20",
                KeyLabel::F21 => "F21",
                KeyLabel::F22 => "F22",
                KeyLabel::F23 => "F23",
                KeyLabel::F24 => "F24",
                KeyLabel::NumLock => "Num Lock",
                KeyLabel::NumPad1 => "Numpad 1",
                KeyLabel::NumPad2 => "Numpad 2",
                KeyLabel::NumPad3 => "Numpad 3",
                KeyLabel::NumPad4 => "Numpad 4",
                KeyLabel::NumPad5 => "Numpad 5",
                KeyLabel::NumPad6 => "Numpad 6",
                KeyLabel::NumPad7 => "Numpad 7",
                KeyLabel::NumPad8 => "Numpad 8",
                KeyLabel::NumPad9 => "Numpad 9",
                KeyLabel::NumPad0 => "Numpad 0",
                KeyLabel::NumPadPlus => "Numpad +",
                KeyLabel::NumPadMinus => "Numpad -",
                KeyLabel::NumPadMultiply => "Numpad *",
                KeyLabel::NumPadDivide => "Numpad /",
                KeyLabel::NumPadEnter => "Numpad Enter",
                KeyLabel::LeftCtrl => "Left Ctrl",
                KeyLabel::LeftShift => "Left Shift",
                KeyLabel::LeftAlt => "Left Alt",
                KeyLabel::RightCtrl => "Right Ctrl",
                KeyLabel::RightShift => "Right Shift",
                KeyLabel::RightAlt => "Right Alt",
                KeyLabel::Up => "Up",
                KeyLabel::Down => "Down",
                KeyLabel::Left => "Left",
                KeyLabel::Right => "Right",
                KeyLabel::Ampersand => "&",
                KeyLabel::Asterisk => "*",
                KeyLabel::At => "@",
                KeyLabel::Backquote => "`",
                KeyLabel::Backslash => "\\",
                KeyLabel::Backspace => "Backspace",
                KeyLabel::CapsLock => "Caps Lock",
                KeyLabel::Caret => "^",
                KeyLabel::Colon => ":",
                KeyLabel::Comma => ",",
                KeyLabel::Delete => "Delete",
                KeyLabel::Dollar => "$",
                KeyLabel::DoubleQuote => "\"",
                KeyLabel::End => "End",
                KeyLabel::Enter => "Enter",
                KeyLabel::Equals => "=",
                KeyLabel::Escape => "Escape",
                KeyLabel::Exclaim => "!",
                KeyLabel::GreaterThan => ">",
                KeyLabel::Hash => "#",
                KeyLabel::Home => "Home",
                KeyLabel::Insert => "Insert",
                KeyLabel::LeftBracket => "[",
                KeyLabel::LeftParen => "(",
                KeyLabel::LessThan => "<",
                KeyLabel::Minus => "-",
                KeyLabel::PageDown => "Page Down",
                KeyLabel::PageUp => "Page Up",
                KeyLabel::Pause => "Pause",
                KeyLabel::Percent => "%",
                KeyLabel::Period => ".",
                KeyLabel::Plus => "+",
                KeyLabel::PrintScreen => "Print Screen",
                KeyLabel::Question => "?",
                KeyLabel::Quote => "'",
                KeyLabel::RightBracket => "]",
                KeyLabel::RightParen => ")",
                KeyLabel::ScrollLock => "Scroll Lock",
                KeyLabel::Semicolon => ";",
                KeyLabel::Slash => "/",
                KeyLabel::Space => "Space",
                KeyLabel::Tab => "Tab",
                KeyLabel::Underscore => "_",
            }
        )
    }
}

/// A key modifier on the keyboard.
///
/// This is primarily useful for creating native-style keyboard shortcuts.
///
/// This type and the associated functions take into account the user's keyboard layout (and
/// any OS-level key mappings). Therefore, the behaviour should match what the user expects
/// for their system.
///
/// For keyboard mappings that are based on position rather than layout, consider using
/// [`Key`] instead.
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
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
}

impl Display for KeyModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                KeyModifier::Ctrl => "Ctrl",
                KeyModifier::Alt => "Alt",
                KeyModifier::Shift => "Shift",
            }
        )
    }
}

#[derive(Default, Debug)]
pub(crate) struct KeyModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
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

/// Returns true if the specified key modifier is currently down.
pub fn is_key_modifier_down(ctx: &Context, key_modifier: KeyModifier) -> bool {
    match key_modifier {
        KeyModifier::Ctrl => ctx.input.key_modifier_state.ctrl,
        KeyModifier::Alt => ctx.input.key_modifier_state.alt,
        KeyModifier::Shift => ctx.input.key_modifier_state.shift,
    }
}

/// Returns true if the specified key modifier is currently up.
pub fn is_key_modifier_up(ctx: &Context, key_modifier: KeyModifier) -> bool {
    match key_modifier {
        KeyModifier::Ctrl => !ctx.input.key_modifier_state.ctrl,
        KeyModifier::Alt => !ctx.input.key_modifier_state.alt,
        KeyModifier::Shift => !ctx.input.key_modifier_state.shift,
    }
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

/// Returns the key that has the specified label in the current keyboard layout.
///
/// For example, passing `KeyLabel::Q` to this function will return different results
/// depending on the active layout:
///
/// * QWERTY: `Some(Key::Q)`
/// * AZERTY: `Some(Key::A)`
/// * Dvorak: `Some(Key::X)`
///
/// If the label is not present in the current keyboard layout, this function will
/// return `None`.
///
/// To convert in the opposite direction (`Key` to `KeyLabel`), use [`get_key_label`].
pub fn get_key_with_label(ctx: &Context, key_label: KeyLabel) -> Option<Key> {
    ctx.window.get_key_with_label(key_label)
}

/// Returns the label for the specified key in the current keyboard layout.
///
/// For example, passing `Key::Q` to this function will return different results
/// depending on the active layout:
///
/// * QWERTY: `Some(KeyLabel::Q)`
/// * AZERTY: `Some(KeyLabel::A)`
/// * Dvorak: `Some(KeyLabel::Quote)`
///
/// If the key cannot be mapped to the current keyboard layout, this function will
/// return `None`.
///
/// To convert in the opposite direction (`KeyLabel` to `Key`), use [`get_key_with_label`].
pub fn get_key_label(ctx: &Context, physical_key: Key) -> Option<KeyLabel> {
    ctx.window.get_key_label(physical_key)
}

pub(crate) fn set_key_down(ctx: &mut Context, key: Key) -> bool {
    let was_up = ctx.input.keys_down.insert(key);

    if was_up || ctx.window.is_key_repeat_enabled() {
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

pub(crate) fn set_key_modifier_state(ctx: &mut Context, state: KeyModifierState) {
    ctx.input.key_modifier_state = state;
}
