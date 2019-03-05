//! Functions and types relating to error handling.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

use image::ImageError;
use rodio::decoder::DecoderError;
use sdl2::video::WindowBuildError;
use sdl2::IntegerOrSdlError;

/// A specialized `Result` type for Tetra.
///
/// All Tetra functions with a recoverable failure condition will return this type.
/// In your game code, you can either use it directly, or wrap it in your own error type.
pub type Result<T = ()> = result::Result<T, TetraError>;

/// Represents the types of error that can occur in a Tetra game.
///
/// Note that if you `match` on this enum, you will be forced to add a wildcard arm by the compiler.
/// This is so that if a new error type is added later on, it will not break your code.
#[derive(Debug)]
pub enum TetraError {
    /// An error that occurred while performing an I/O operation (e.g. while loading a file).
    Io(io::Error),

    /// An error that was returned by SDL.
    Sdl(String),

    /// An error that was returned by OpenGL.
    OpenGl(String),

    /// An error that occured while processing an image.
    Image(ImageError),

    /// Returned when not enough data is provided to fill a buffer.
    /// This may happen if you're creating a texture from raw data and you don't provide
    /// enough data.
    NotEnoughData {
        /// The number of bytes that were expected.
        expected: usize,

        /// The number of bytes that were provided.
        actual: usize,
    },

    /// Returned when trying to play back audio without an available device.
    NoAudioDevice,

    /// An error that occured while decoding audio data.
    FailedToDecodeAudio(DecoderError),

    /// This is here so that adding new error types will not be a breaking change.
    /// Can be removed once #[non_exhaustive] is stabilized.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Display for TetraError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TetraError::Io(e) => write!(f, "IO error: {}", e),
            TetraError::Sdl(e) => write!(f, "SDL error: {}", e),
            TetraError::OpenGl(e) => write!(f, "OpenGL error: {}", e),
            TetraError::Image(e) => write!(f, "Image processing error: {}", e),
            TetraError::NotEnoughData { expected, actual } => write!(
                f,
                "Not enough data was provided to fill a buffer - expected {}, found {}.",
                expected, actual
            ),
            TetraError::NoAudioDevice => write!(f, "No audio device was available for playback."),
            TetraError::FailedToDecodeAudio(e) => write!(f, "Failed to decode audio: {}", e),
            TetraError::__Nonexhaustive => unreachable!(),
        }
    }
}

impl Error for TetraError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TetraError::Io(e) => Some(e),
            TetraError::Sdl(_) => None,
            TetraError::OpenGl(_) => None,
            TetraError::Image(e) => Some(e),
            TetraError::NotEnoughData { .. } => None,
            TetraError::NoAudioDevice => None,
            TetraError::FailedToDecodeAudio(e) => Some(e),
            TetraError::__Nonexhaustive => unreachable!(),
        }
    }
}

impl From<io::Error> for TetraError {
    fn from(e: io::Error) -> TetraError {
        TetraError::Io(e)
    }
}

impl From<ImageError> for TetraError {
    fn from(e: ImageError) -> TetraError {
        TetraError::Image(e)
    }
}

impl From<WindowBuildError> for TetraError {
    fn from(e: WindowBuildError) -> TetraError {
        TetraError::Sdl(e.to_string())
    }
}

impl From<IntegerOrSdlError> for TetraError {
    fn from(e: IntegerOrSdlError) -> TetraError {
        TetraError::Sdl(e.to_string())
    }
}

impl From<DecoderError> for TetraError {
    fn from(e: DecoderError) -> TetraError {
        TetraError::FailedToDecodeAudio(e)
    }
}
