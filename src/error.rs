//! Functions and types relating to error handling.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::result;

use image::ImageError;

use crate::platform::DecoderError;

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
    Fatal {
        reason: String,
    },

    FailedToLoadAsset {
        reason: io::Error,
        path: PathBuf,
    },

    InvalidTexture {
        reason: ImageError,
    },

    InvalidShader {
        reason: String,
    },

    InvalidSound {
        reason: DecoderError,
    },

    FailedToChangeDisplayMode {
        reason: String,
    },

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

    /// This is here so that adding new error types will not be a breaking change.
    /// Can be removed once #[non_exhaustive] is stabilized.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Display for TetraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TetraError::Fatal { reason } => write!(f, "Platform error: {}", reason),
            TetraError::FailedToLoadAsset { reason, path } => write!(
                f,
                "Failed to load asset from {}: {}",
                path.to_string_lossy(),
                reason
            ),
            TetraError::InvalidTexture { reason } => write!(f, "Invalid texture: {}", reason),
            TetraError::InvalidShader { reason } => write!(f, "Invalid shader: {}", reason),
            TetraError::InvalidSound { reason } => write!(f, "Invalid sound: {}", reason),
            TetraError::FailedToChangeDisplayMode { reason } => {
                write!(f, "Failed to change display mode: {}", reason)
            }
            TetraError::NotEnoughData { expected, actual } => write!(
                f,
                "Not enough data was provided to fill a buffer - expected {}, found {}.",
                expected, actual
            ),
            TetraError::NoAudioDevice => write!(f, "No audio device was available for playback."),
            TetraError::__Nonexhaustive => unreachable!(),
        }
    }
}

impl Error for TetraError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TetraError::Fatal { .. } => None,
            TetraError::FailedToLoadAsset { reason, .. } => Some(reason),
            TetraError::InvalidTexture { reason } => Some(reason),
            TetraError::InvalidShader { .. } => None,
            TetraError::InvalidSound { reason } => Some(reason),
            TetraError::FailedToChangeDisplayMode { .. } => None,
            TetraError::NotEnoughData { .. } => None,
            TetraError::NoAudioDevice => None,
            TetraError::__Nonexhaustive => unreachable!(),
        }
    }
}
