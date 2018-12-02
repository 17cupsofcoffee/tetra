//! Functions and types relating to error handling.

/// A specialized `Result` type for Tetra.
///
/// All Tetra functions with a recoverable failure condition will return this type.
/// In your game code, you can either use it directly, or wrap it in your own error type.
pub type Result<T = ()> = std::result::Result<T, TetraError>;

/// Represents the types of error that can occur in a Tetra game.
#[derive(Debug)]
pub enum TetraError {
    /// An error that occurred while performing an I/O operation (e.g. while loading a file).
    Io(std::io::Error),

    /// An error that was returned by SDL.
    Sdl(String),

    /// An error that was returned by OpenGL.
    OpenGl(String),

    /// An error that occured while processing an image.
    Image(image::ImageError),
}
