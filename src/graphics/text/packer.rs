use crate::error::Result;
use crate::graphics::{FilterMode, Texture};
use crate::platform::GraphicsDevice;

/// An individual shelf within the packed atlas, tracking how much space
/// is currently taken up.
#[derive(Copy, Clone, Debug)]
struct Shelf {
    current_x: i32,
    start_y: i32,
    height: i32,
}

/// Packs texture data into an atlas using a naive shelf-packing algorithm.
pub struct ShelfPacker {
    texture: Texture,
    shelves: Vec<Shelf>,
    next_y: i32,
}

impl ShelfPacker {
    const PADDING: i32 = 1;

    /// Creates a new `ShelfPacker`.
    pub fn new(
        device: &mut GraphicsDevice,
        texture_width: i32,
        texture_height: i32,
    ) -> Result<ShelfPacker> {
        Ok(ShelfPacker {
            texture: Texture::with_device_empty(
                device,
                texture_width,
                texture_height,
                FilterMode::Nearest,
            )?,
            shelves: Vec::new(),
            next_y: Self::PADDING,
        })
    }

    /// Returns a reference to the current atlas texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Resize the atlas texture, clearing any existing shelf data.
    pub fn resize(
        &mut self,
        device: &mut GraphicsDevice,
        texture_width: i32,
        texture_height: i32,
    ) -> Result {
        self.texture =
            Texture::with_device_empty(device, texture_width, texture_height, FilterMode::Nearest)?;

        self.shelves.clear();
        self.next_y = 0;

        Ok(())
    }

    /// Tries to insert RGBA data into the atlas, and returns the position.
    ///
    /// If the data will not fit into the remaining space, `None` will be returned.
    pub fn insert(
        &mut self,
        device: &mut GraphicsDevice,
        data: &[u8],
        width: i32,
        height: i32,
    ) -> Option<(i32, i32)> {
        let space = self.find_space(width, height);

        if let Some((x, y)) = space {
            device.set_texture_data(&self.texture.handle.borrow(), data, x, y, width, height);
        }

        space
    }

    /// Finds a space in the atlas that can fit a sprite of the specified width and height,
    /// and returns the position.
    ///
    /// If it would not fit into the remaining space, `None` will be returned.
    fn find_space(&mut self, source_width: i32, source_height: i32) -> Option<(i32, i32)> {
        let texture_width = self.texture.width();
        let texture_height = self.texture.height();

        self.shelves
            .iter_mut()
            .find(|shelf| {
                shelf.height >= source_height
                    && texture_width - shelf.current_x - Self::PADDING >= source_width
            })
            .map(|shelf| {
                // Use existing shelf:
                let position = (shelf.current_x, shelf.start_y);
                shelf.current_x += source_width + Self::PADDING;
                position
            })
            .or_else(|| {
                if self.next_y + source_height < texture_height {
                    // Create new shelf:
                    let position = (Self::PADDING, self.next_y);

                    self.shelves.push(Shelf {
                        current_x: source_width + Self::PADDING * 2,
                        start_y: self.next_y,
                        height: source_height,
                    });

                    self.next_y += source_height + Self::PADDING;

                    Some(position)
                } else {
                    // Won't fit:
                    None
                }
            })
    }
}
