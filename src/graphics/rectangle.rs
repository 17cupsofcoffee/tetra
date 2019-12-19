use crate::math::Vec2;

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

    /// Returns `true` if the `other` rectangle intersects with `self`.
    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns `true` if the `other` rectangle is fully contained within `self`.
    pub fn contains(&self, other: &Rectangle) -> bool {
        self.x <= other.x
            && other.x + other.width <= self.x + self.width
            && self.y <= other.y
            && other.y + other.height <= self.y + self.height
    }

    /// Returns `true` if the provided point is within the bounds of `self`.
    pub fn contains_point(&self, point: Vec2<f32>) -> bool {
        self.x <= point.x
            && point.x < self.x + self.width
            && self.y <= point.y
            && point.y < self.y + self.height
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

#[cfg(test)]
mod tests {
    use super::{Rectangle, Vec2};

    #[test]
    fn intersects() {
        let base = Rectangle::new(2.0, 2.0, 4.0, 4.0);
        let fully_contained = Rectangle::new(2.5, 2.5, 2.0, 2.0);
        let overlapping = Rectangle::new(3.0, 3.0, 4.0, 4.0);
        let seperate = Rectangle::new(20.0, 20.0, 4.0, 4.0);
        let adjacent = Rectangle::new(6.0, 2.0, 4.0, 4.0);

        assert!(base.intersects(&base));
        assert!(base.intersects(&fully_contained));
        assert!(base.intersects(&overlapping));

        assert!(!base.intersects(&seperate));
        assert!(!base.intersects(&adjacent));
    }

    #[test]
    fn contains() {
        let base = Rectangle::new(2.0, 2.0, 4.0, 4.0);
        let fully_contained = Rectangle::new(2.5, 2.5, 2.0, 2.0);
        let overlapping = Rectangle::new(3.0, 3.0, 4.0, 4.0);
        let seperate = Rectangle::new(20.0, 20.0, 4.0, 4.0);
        let adjacent = Rectangle::new(6.0, 2.0, 4.0, 4.0);

        assert!(base.contains(&base));
        assert!(base.contains(&fully_contained));

        assert!(!base.contains(&overlapping));
        assert!(!base.contains(&seperate));
        assert!(!base.contains(&adjacent));
    }

    #[test]
    fn contains_point() {
        let base = Rectangle::new(2.0, 2.0, 4.0, 4.0);

        let top_left = Vec2::new(2.0, 2.0);
        let top_right = Vec2::new(6.0, 2.0);
        let bottom_left = Vec2::new(2.0, 6.0);
        let bottom_right = Vec2::new(6.0, 6.0);
        let centre = Vec2::new(4.0, 4.0);
        let less_than = Vec2::new(1.0, 1.0);
        let more_than = Vec2::new(7.0, 7.0);

        assert!(base.contains_point(top_left));
        assert!(base.contains_point(centre));

        assert!(!base.contains_point(top_right));
        assert!(!base.contains_point(bottom_left));
        assert!(!base.contains_point(bottom_right));
        assert!(!base.contains_point(less_than));
        assert!(!base.contains_point(more_than));
    }
}
