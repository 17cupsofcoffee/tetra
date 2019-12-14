/// A rectangle of `f32`s.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Rectangle {
    /// The X co-ordinate of the rectangle.
    pub x: f32,

    /// The Y co-ordinate of the rectangle.
    pub y: f32,

    /// The width of the rectangle.
    pub width: f32,

    /// The height of the rectangle.
    pub height: f32,
}

impl Rectangle {
    /// Creates a new `Rectangle`.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns an infinite iterator of horizontally adjecent rectangles, starting at the specified
    /// point and increasing along the X axis.
    ///
    /// This can be useful when slicing spritesheets.
    ///
    /// # Examples
    /// ```
    /// # use tetra::graphics::Rectangle;
    /// let rects: Vec<Rectangle> = Rectangle::row(0.0, 0.0, 16.0, 16.0).take(3).collect();
    ///
    /// assert_eq!(Rectangle::new(0.0, 0.0, 16.0, 16.0), rects[0]);
    /// assert_eq!(Rectangle::new(16.0, 0.0, 16.0, 16.0), rects[1]);
    /// assert_eq!(Rectangle::new(32.0, 0.0, 16.0, 16.0), rects[2]);
    /// ```
    pub fn row(x: f32, y: f32, width: f32, height: f32) -> impl Iterator<Item = Rectangle> {
        RectangleRow {
            next_rect: Rectangle::new(x, y, width, height),
        }
    }

    /// Returns an infinite iterator of vertically adjecent rectangles, starting at the specified
    /// point and increasing along the Y axis.
    ///
    /// This can be useful when slicing spritesheets.
    ///
    /// # Examples
    /// ```
    /// # use tetra::graphics::Rectangle;
    /// let rects: Vec<Rectangle> = Rectangle::column(0.0, 0.0, 16.0, 16.0).take(3).collect();
    ///
    /// assert_eq!(Rectangle::new(0.0, 0.0, 16.0, 16.0), rects[0]);
    /// assert_eq!(Rectangle::new(0.0, 16.0, 16.0, 16.0), rects[1]);
    /// assert_eq!(Rectangle::new(0.0, 32.0, 16.0, 16.0), rects[2]);
    /// ```
    pub fn column(x: f32, y: f32, width: f32, height: f32) -> impl Iterator<Item = Rectangle> {
        RectangleColumn {
            next_rect: Rectangle::new(x, y, width, height),
        }
    }
}

#[derive(Debug, Clone)]
struct RectangleRow {
    next_rect: Rectangle,
}

impl Iterator for RectangleRow {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        let current_rect = self.next_rect;
        self.next_rect.x += self.next_rect.width;
        Some(current_rect)
    }
}

#[derive(Debug, Clone)]
struct RectangleColumn {
    next_rect: Rectangle,
}

impl Iterator for RectangleColumn {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        let current_rect = self.next_rect;
        self.next_rect.y += self.next_rect.height;
        Some(current_rect)
    }
}
