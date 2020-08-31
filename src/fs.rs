//! Internal utilities for reading files while retaining context about file paths.

// To avoid warnings in the rare case where all features are disabled at the same time:
#![allow(unused)]

use std::fs;
use std::path::Path;

use image::{self, DynamicImage, ImageError};

use crate::error::{Result, TetraError};

pub(crate) fn read<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    fs::read(path_ref).map_err(|e| TetraError::FailedToLoadAsset {
        reason: e,
        path: path_ref.to_owned(),
    })
}

pub(crate) fn read_to_image<P>(path: P) -> Result<DynamicImage>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    image::open(path_ref).map_err(|e| match e {
        ImageError::IoError(inner) => TetraError::FailedToLoadAsset {
            reason: inner,
            path: path_ref.to_owned(),
        },
        _ => TetraError::InvalidTexture(e),
    })
}

pub(crate) fn read_to_string<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    fs::read_to_string(path_ref).map_err(|e| TetraError::FailedToLoadAsset {
        reason: e,
        path: path_ref.to_owned(),
    })
}
