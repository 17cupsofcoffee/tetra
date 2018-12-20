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

impl From<std::io::Error> for TetraError {
    fn from(io_error: std::io::Error) -> TetraError {
        TetraError::Io(io_error)
    }
}

impl From<image::ImageError> for TetraError {
    fn from(image_error: image::ImageError) -> TetraError {
        TetraError::Image(image_error)
    }
}

impl From<sdl2::video::WindowBuildError> for TetraError {
    fn from(window_build_error: sdl2::video::WindowBuildError) -> TetraError {
        TetraError::Sdl(window_build_error.to_string())
    }
}

impl From<sdl2::IntegerOrSdlError> for TetraError {
    fn from(integer_or_sdl_error: sdl2::IntegerOrSdlError) -> TetraError {
        TetraError::Sdl(integer_or_sdl_error.to_string())
    }
}
