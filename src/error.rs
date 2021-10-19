//! Functions and types relating to error handling.

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::result;

use image::ImageError;

use lyon_tessellation::{InternalError, TessellationError};

#[cfg(feature = "audio")]
use rodio::decoder::DecoderError;

/// A specialized [`Result`](std::result::Result) type for Tetra.
///
/// All Tetra functions with a recoverable failure condition will return this type.
/// In your game code, you can either use it directly, or wrap it in your own error type.
pub type Result<T = ()> = result::Result<T, TetraError>;

/// The types of error that can occur in a Tetra game.
#[non_exhaustive]
#[derive(Debug)]
pub enum TetraError {
    /// Returned when the underlying platform returns an unexpected error.
    /// This usually isn't something your game can reasonably be expected to recover from.
    PlatformError(String),

    /// Returned when your game fails to load an asset. This is usually caused by an
    /// incorrect file path, or some form of permission issues.
    FailedToLoadAsset {
        /// The underlying reason for the error.
        reason: io::Error,

        /// The path to the asset that failed to load.
        path: PathBuf,
    },

    /// Returned when a color is invalid.
    InvalidColor,

    /// Returned when a texture's data is invalid.
    InvalidTexture(ImageError),

    /// Returned when a shader fails to compile.
    InvalidShader(String),

    /// Returned when a font could not be read.
    InvalidFont,

    /// Returned when a sound cannot be decoded.
    #[cfg(feature = "audio")]
    InvalidSound(DecoderError),

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

    /// Returned when your game tried to change the display settings (e.g. fullscreen, vsync)
    /// but was unable to do so.
    FailedToChangeDisplayMode(String),

    /// Returned when a shape cannot be tessellated.
    TessellationError(TessellationError),
}

impl Display for TetraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TetraError::PlatformError(msg) => {
                write!(f, "An error was thrown by the platform: {}", msg)
            }
            TetraError::FailedToLoadAsset { path, .. } => {
                write!(f, "Failed to load asset from {}", path.to_string_lossy())
            }
            TetraError::InvalidColor => write!(f, "Invalid color"),
            TetraError::InvalidTexture(_) => write!(f, "Invalid texture data"),
            TetraError::InvalidShader(msg) => write!(f, "Invalid shader source: {}", msg),
            TetraError::InvalidFont => write!(f, "Invalid font data"),
            #[cfg(feature = "audio")]
            TetraError::InvalidSound(_) => write!(f, "Invalid sound data"),
            TetraError::NotEnoughData { expected, actual } => write!(
                f,
                "Not enough data was provided to fill a buffer - expected {}, found {}.",
                expected, actual
            ),
            TetraError::FailedToChangeDisplayMode(msg) => {
                write!(f, "Failed to change display mode: {}", msg)
            }
            TetraError::NoAudioDevice => write!(f, "No audio device available for playback"),
            TetraError::TessellationError(e) => {
                write!(
                    f,
                    "An error occurred while tessellating a shape: {}",
                    tess_error_description(e)
                )
            }
        }
    }
}

impl Error for TetraError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TetraError::PlatformError(_) => None,
            TetraError::FailedToLoadAsset { reason, .. } => Some(reason),
            TetraError::InvalidColor => None,
            TetraError::InvalidTexture(reason) => Some(reason),
            TetraError::InvalidShader(_) => None,
            TetraError::InvalidFont => None,
            #[cfg(feature = "audio")]
            TetraError::InvalidSound(reason) => Some(reason),
            TetraError::NotEnoughData { .. } => None,
            TetraError::NoAudioDevice => None,
            TetraError::FailedToChangeDisplayMode(_) => None,

            // This should return the inner error, but Lyon doesn't implement Error for some reason,
            // so we can't :(
            TetraError::TessellationError(_) => None,
        }
    }
}

fn tess_error_description(err: &TessellationError) -> Cow<'static, str> {
    match err {
        TessellationError::UnsupportedParamater => Cow::Borrowed("unsupported parameter"),
        TessellationError::InvalidVertex => Cow::Borrowed("invalid vertex"),
        TessellationError::TooManyVertices => Cow::Borrowed("too many vertices"),
        TessellationError::Internal(internal_err) => match internal_err {
            InternalError::IncorrectActiveEdgeOrder(code) => {
                Cow::Owned(format!("incorrect active edge order {}", code))
            }
            InternalError::InsufficientNumberOfSpans => {
                Cow::Borrowed("insufficient number of spans")
            }
            InternalError::InsufficientNumberOfEdges => {
                Cow::Borrowed("insufficient number of edges")
            }
            InternalError::MergeVertexOutside => Cow::Borrowed("merge vertex outside"),
            InternalError::InvalidNumberOfEdgesBelowVertex => {
                Cow::Borrowed("invalid number of edges below vertex")
            }
            InternalError::ErrorCode(code) => Cow::Owned(format!("error code {}", code)),
        },
    }
}
