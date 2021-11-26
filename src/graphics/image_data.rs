use std::path::Path;

use image::{Rgba, RgbaImage, SubImage};

use crate::error::{Result, TetraError};
use crate::fs;
use crate::graphics::{Color, Rectangle, Texture};
use crate::math::Vec2;
use crate::Context;

/// Raw image data.
///
/// # Supported Formats
///
/// Various file formats are supported, and can be enabled or disabled via Cargo features:
///
/// | Format | Cargo feature | Enabled by default? |
/// |-|-|-|
/// | PNG | `texture_png` | Yes |
/// | JPEG | `texture_jpeg` | Yes |
/// | GIF | `texture_gif` | Yes |
/// | BMP | `texture_bmp` | Yes |
/// | TIFF | `texture_tiff` | No |
/// | TGA | `texture_tga` | No |
/// | WebP | `texture_webp` | No |
/// | ICO | `texture_ico` | No |
/// | PNM | `texture_pnm` | No |
/// | DDS/DXT | `texture_dds` | No |
#[derive(Debug, Clone)]
pub struct ImageData {
    data: RgbaImage,
}

impl ImageData {
    /// Loads image data from the given file.
    ///
    /// The format will be determined based on the file extension.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if the file could not be loaded.
    /// * [`TetraError::InvalidTexture`] will be returned if the image data was invalid.
    pub fn new<P>(path: P) -> Result<ImageData>
    where
        P: AsRef<Path>,
    {
        Ok(ImageData {
            data: fs::read_to_image(path)?.into_rgba8(),
        })
    }

    /// Creates an `ImageData` from raw RGBA8 data.
    ///
    /// This function takes `Into<Vec<u8>>`. If you pass a `Vec<u8>`, that `Vec` will
    /// be reused for the created `ImageData` without reallocating. Otherwise, the data
    /// will be copied.
    ///
    /// This function requires you to provide enough data to fill the image's bounds.
    /// If you provide too little data, an error will be returned.
    /// If you provide too much data, it will be truncated.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NotEnoughData`] will be returned if not enough data is provided to fill
    /// the image.
    pub fn from_data<D>(width: i32, height: i32, data: D) -> Result<ImageData>
    where
        D: Into<Vec<u8>>,
    {
        // TODO: Add texture format support before 0.7 release

        let data = data.into();

        let expected = (width * height * 4) as usize;
        let actual = data.len();

        if actual < expected {
            return Err(TetraError::NotEnoughData { expected, actual });
        }

        let image = RgbaImage::from_vec(width as u32, height as u32, data).unwrap();

        Ok(ImageData { data: image })
    }

    /// Decodes image data that is encoded in one of Tetra's supported
    /// file formats (except for TGA).
    ///
    /// This is useful in combination with [`include_bytes`](std::include_bytes), as it
    /// allows you to include your image data directly in the binary.
    ///
    /// The format will be determined based on the 'magic bytes' at the beginning of the
    /// data. Note that TGA files do not have recognizable magic bytes, so this function
    /// will not recognize them.
    ///
    /// # Errors
    ///
    /// * [`TetraError::InvalidTexture`] will be returned if the image data was invalid.
    pub fn from_encoded(data: &[u8]) -> Result<ImageData> {
        let image = image::load_from_memory(data)
            .map_err(TetraError::InvalidTexture)?
            .into_rgba8();

        Ok(ImageData { data: image })
    }

    /// Returns the width of the image.
    pub fn width(&self) -> i32 {
        self.data.width() as i32
    }

    /// Returns the height of the image.
    pub fn height(&self) -> i32 {
        self.data.height() as i32
    }

    /// Returns the size of the image.
    pub fn size(&self) -> (i32, i32) {
        let (width, height) = self.data.dimensions();
        (width as i32, height as i32)
    }

    /// Returns the image's data, as a slice of raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns the image's data, as a mutable slice of raw bytes.
    ///
    /// This is not currently exposed publicly, as some more thought is needed
    /// into whether doing so would cause issues once different pixel formats
    /// are supported.
    pub(crate) fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns the image's underlying buffer.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data.into_raw()
    }

    /// Creates a new `ImageData` from a region.
    ///
    /// This will copy the data into a new buffer - as such, calling this function
    /// can be expensive!
    pub fn region(&self, region: Rectangle<i32>) -> ImageData {
        let subimage = SubImage::new(
            &self.data,
            region.x as u32,
            region.y as u32,
            region.width as u32,
            region.height as u32,
        );

        let data = subimage.to_image();

        ImageData { data }
    }

    /// Creates a new [`Texture`] from the stored data.
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`] will be returned if the underlying graphics API encounters an error.
    pub fn to_texture(&self, ctx: &mut Context) -> Result<Texture> {
        Texture::from_image_data(ctx, self)
    }

    /// Gets the color of the pixel at the specified location.
    ///
    /// # Panics
    ///
    /// Panics if the location is outside the bounds of the image.
    pub fn get_pixel_color(&self, position: Vec2<i32>) -> Color {
        let pixel = self.data.get_pixel(position.x as u32, position.y as u32).0;
        pixel.into()
    }

    /// Sets the color of the pixel at the specified location.
    ///
    /// # Panics
    ///
    /// Panics if the location is outside the bounds of the image.
    pub fn set_pixel_color(&mut self, position: Vec2<i32>, color: Color) {
        self.data
            .put_pixel(position.x as u32, position.y as u32, Rgba(color.into()));
    }

    /// Transforms the image data by applying a function to each pixel.
    pub fn transform<F>(&mut self, mut func: F)
    where
        F: FnMut(Vec2<i32>, Color) -> Color,
    {
        for (x, y, pixel) in self.data.enumerate_pixels_mut() {
            let output = func(Vec2::new(x as i32, y as i32), pixel.0.into());
            *pixel = Rgba(output.into());
        }
    }

    /// Multiplies the RGB components of each pixel by the alpha component.
    ///
    /// This can be useful when working with
    /// [premultiplied alpha blending](super::BlendState::alpha).
    pub fn premultiply(&mut self) {
        self.transform(|_, color| color.to_premultiplied())
    }
}
