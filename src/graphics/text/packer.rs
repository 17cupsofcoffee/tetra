use crate::graphics::{FilterMode, Rectangle, Texture};
use crate::platform::GraphicsDevice;
use crate::{Context, Result};

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
    /// Creates a new `ShelfPacker`.
    pub fn new(
        device: &mut GraphicsDevice,
        texture_width: i32,
        texture_height: i32,
        filter_mode: FilterMode,
    ) -> Result<ShelfPacker> {
        Ok(ShelfPacker {
            texture: Texture::with_device_empty(
                device,
                texture_width,
                texture_height,
                filter_mode,
            )?,
            shelves: Vec::new(),
            next_y: 0,
        })
    }

    /// Returns a reference to the current atlas texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn filter_mode(&self) -> FilterMode {
        self.texture.filter_mode()
    }

    pub fn set_filter_mode(&mut self, ctx: &mut Context, filter_mode: FilterMode) {
        self.texture.set_filter_mode(ctx, filter_mode);
    }

    /// Resize the atlas texture, clearing any existing shelf data.
    pub fn resize(
        &mut self,
        device: &mut GraphicsDevice,
        texture_width: i32,
        texture_height: i32,
    ) -> Result {
        self.texture = Texture::with_device_empty(
            device,
            texture_width,
            texture_height,
            self.texture.filter_mode(),
        )?;

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
        padding: i32,
    ) -> Option<Rectangle<i32>> {
        let padded_width = width + padding * 2;
        let padded_height = height + padding * 2;

        let space = self.find_space(padded_width, padded_height);

        if let Some(s) = space {
            device
                .set_texture_data(
                    &self.texture.data.handle,
                    data,
                    s.x + padding,
                    s.y + padding,
                    width,
                    height,
                )
                .expect("glyph packer should never write out of bounds");
        }

        space
    }

    /// Finds a space in the atlas that can fit a sprite of the specified width and height,
    /// and returns the position.
    ///
    /// If it would not fit into the remaining space, `None` will be returned.
    fn find_space(&mut self, source_width: i32, source_height: i32) -> Option<Rectangle<i32>> {
        let texture_width = self.texture.width();
        let texture_height = self.texture.height();

        self.shelves
            .iter_mut()
            .find(|shelf| {
                shelf.height >= source_height && texture_width - shelf.current_x >= source_width
            })
            .map(|shelf| {
                // Use existing shelf:
                let position = (shelf.current_x, shelf.start_y);
                shelf.current_x += source_width;

                Rectangle::new(position.0, position.1, source_width, source_height)
            })
            .or_else(|| {
                if self.next_y + source_height < texture_height {
                    // Create new shelf:
                    let position = (0, self.next_y);

                    self.shelves.push(Shelf {
                        current_x: source_width,
                        start_y: self.next_y,
                        height: source_height,
                    });

                    self.next_y += source_height;

                    Some(Rectangle::new(
                        position.0,
                        position.1,
                        source_width,
                        source_height,
                    ))
                } else {
                    // Won't fit:
                    None
                }
            })
    }
}
