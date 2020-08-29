use std::path::PathBuf;

use crate::input::{GamepadAxis, GamepadButton, GamepadStick, Key, MouseButton};
use crate::math::Vec2;
use crate::{Context, Result};

/// A trait representing a type that contains game state and provides logic for updating it
/// and drawing it to the screen. This is where you'll write your game logic!
///
/// The methods on `State` allow you to return a `Result`, either explicitly or via the `?`
/// operator. If an error is returned, the game will close and the error will be returned from
/// the `run` function that was used to start it.
#[allow(unused_variables)]
pub trait State {
    /// Called when it is time for the game to update.
    fn update(&mut self, ctx: &mut Context) -> Result {
        Ok(())
    }

    /// Called when it is time for the game to be drawn.
    fn draw(&mut self, ctx: &mut Context) -> Result {
        Ok(())
    }

    /// Called when a window or input event occurs.
    fn event(&mut self, ctx: &mut Context, event: Event) -> Result {
        Ok(())
    }
}

/// Events that can occur while the game is running.
///
/// The [`event` method on the `State` trait](trait.State.html#method.event) will recieve
/// events as they occur.
///
/// # Examples
///
/// The [`events`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/events.rs)
/// example demonstrates how to handle events.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Event {
    /// The game window was resized.
    Resized {
        /// The new width of the game window.
        width: i32,

        /// The new height of the game window.
        height: i32,
    },

    /// The game window was focused by the user.
    FocusGained,

    /// The game window was un-focused by the user.
    FocusLost,

    /// A key on the keyboard was pressed.
    KeyPressed {
        /// The key that was pressed.
        key: Key,
    },

    /// A key on the keyboard was released.
    KeyReleased {
        /// The key that was released.
        key: Key,
    },

    /// A button on the mouse was pressed.
    MouseButtonPressed {
        /// The button that was pressed.
        button: MouseButton,
    },

    /// A button on the mouse was released.
    MouseButtonReleased {
        /// The button that was released.
        button: MouseButton,
    },

    /// The mouse was moved.
    MouseMoved {
        /// The new position of the mouse, in window co-ordinates.
        ///
        /// If [relative mouse mode](./window/fn.set_relative_mouse_mode.html) is
        /// enabled, this field is not guarenteed to update.
        position: Vec2<f32>,

        /// The movement of the mouse, relative to the `position` of the previous
        /// `MouseMoved` event.
        delta: Vec2<f32>,
    },

    /// The mouse wheel was moved.
    MouseWheelMoved {
        /// The amount that the wheel was moved.
        ///
        /// Most 'normal' mice can only scroll vertically, but some devices can also scroll horizontally.
        /// Use the Y component of the returned vector if you don't care about horizontal scroll.
        ///
        /// Positive values correspond to scrolling up/right, negative values correspond to scrolling
        /// down/left.
        amount: Vec2<i32>,
    },

    /// A gamepad was connected to the system.
    GamepadAdded {
        /// The ID that was assigned to the gamepad.
        id: usize,
    },

    /// A gamepad was removed from the system.
    GamepadRemoved {
        /// The ID of the gamepad that was removed.
        id: usize,
    },

    /// A button on a gamepad was pressed.
    GamepadButtonPressed {
        /// The ID of the gamepad.
        id: usize,

        /// The button that was pressed.
        button: GamepadButton,
    },

    /// A button on a gamepad was released.
    GamepadButtonReleased {
        /// The ID of the gamepad.
        id: usize,

        /// The button that was released.
        button: GamepadButton,
    },

    /// An axis on a gamepad was moved.
    GamepadAxisMoved {
        /// The ID of the gamepad.
        id: usize,

        /// The axis that was moved.
        axis: GamepadAxis,

        /// The new position of the axis.
        position: f32,
    },

    /// A control stick on a gamepad was moved.
    GamepadStickMoved {
        /// The ID of the gamepad.
        id: usize,

        /// The stick that was moved.
        stick: GamepadStick,

        /// The new position of the stick.
        position: Vec2<f32>,
    },

    /// The user typed some text.
    TextInput {
        /// The text that was typed by the user.
        text: String,
    },

    /// The user dropped a file into the window.
    ///
    /// This event will be fired multiple times if the user dropped multiple files at the
    /// same time.
    ///
    /// Note that on MacOS, you must [edit your `info.plist` file to set which document types
    /// you want your application to support](https://help.apple.com/xcode/mac/current/#/devddd273fdd),
    /// otherwise no `FileDropped` events will be fired.
    FileDropped {
        /// The path of the file that was dropped.
        path: PathBuf,
    },
}
