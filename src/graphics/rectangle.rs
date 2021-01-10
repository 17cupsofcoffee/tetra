use std::ops::{Add, AddAssign, Div};

use crate::math::Vec2;

/// A rectangle, represented by a top-left position, a width and a height.
///
/// # Serde
///
/// Serialization and deserialization of this type (via [Serde](https://serde.rs/))
/// can be enabled via the `serde_support` feature.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Rectangle<T = f32> {
    /// The X co-ordinate of the rectangle.
    pub x: T,

    /// The Y co-ordinate of the rectangle.
    pub y: T,

    /// The width of the rectangle.
    pub width: T,

    /// The height of the rectangle.
    pub height: T,
}

impl<T> Rectangle<T> {
    /// Creates a new `Rectangle`.
    pub const fn new(x: T, y: T, width: T, height: T) -> Rectangle<T> {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

impl<T> Rectangle<T>
where
    T: Copy,
{
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
    pub fn row(x: T, y: T, width: T, height: T) -> impl Iterator<Item = Rectangle<T>>
    where
        T: AddAssign,
    {
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
    pub fn column(x: T, y: T, width: T, height: T) -> impl Iterator<Item = Rectangle<T>>
    where
        T: AddAssign,
    {
        RectangleColumn {
            next_rect: Rectangle::new(x, y, width, height),
        }
    }

    /// Returns `true` if the `other` rectangle intersects with `self`.
    pub fn intersects(&self, other: &Rectangle<T>) -> bool
    where
        T: Add<Output = T> + PartialOrd,
    {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns `true` if the `other` rectangle is fully contained within `self`.
    pub fn contains(&self, other: &Rectangle<T>) -> bool
    where
        T: Add<Output = T> + PartialOrd,
    {
        self.x <= other.x
            && other.x + other.width <= self.x + self.width
            && self.y <= other.y
            && other.y + other.height <= self.y + self.height
    }

    /// Returns `true` if the provided point is within the bounds of `self`.
    pub fn contains_point(&self, point: Vec2<T>) -> bool
    where
        T: Add<Output = T> + PartialOrd,
    {
        self.x <= point.x
            && point.x < self.x + self.width
            && self.y <= point.y
            && point.y < self.y + self.height
    }

    /// Returns the X co-ordinate of the left side of the rectangle.
    ///
    /// You can also obtain this via the `x` field - this method is provided for
    /// symmetry with the [`right`](Self::right) method.
    pub fn left(&self) -> T {
        self.x
    }

    /// Returns the X co-ordinate of the right side of the rectangle.
    pub fn right(&self) -> T
    where
        T: Add<Output = T>,
    {
        self.x + self.width
    }

    /// Returns the Y co-ordinate of the top of the rectangle.
    ///
    /// You can also obtain this via the `y` field - this method is provided for
    /// symmetry with the [`bottom`](Self::bottom) method.
    pub fn top(&self) -> T {
        self.y
    }

    /// Returns the Y co-ordinate of the bottom of the rectangle.
    pub fn bottom(&self) -> T
    where
        T: Add<Output = T>,
    {
        self.y + self.height
    }

    /// Returns the co-ordinates of the center point of the rectangle.
    pub fn center(&self) -> Vec2<T>
    where
        T: Add<Output = T> + Div<Output = T> + From<u8>,
    {
        Vec2::new(
            self.x + (self.width / T::from(2)),
            self.y + (self.height / T::from(2)),
        )
    }

    /// Returns the co-ordinates of the top-left point of the rectangle.
    pub fn top_left(&self) -> Vec2<T> {
        Vec2::new(self.x, self.y)
    }

    /// Returns the co-ordinates of the top-right point of the rectangle.
    pub fn top_right(&self) -> Vec2<T>
    where
        T: Add<Output = T>,
    {
        Vec2::new(self.right(), self.y)
    }

    /// Returns the co-ordinates of the bottom-left point of the rectangle.
    pub fn bottom_left(&self) -> Vec2<T>
    where
        T: Add<Output = T>,
    {
        Vec2::new(self.x, self.bottom())
    }

    /// Returns the co-ordinates of the bottom-right point of the rectangle.
    pub fn bottom_right(&self) -> Vec2<T>
    where
        T: Add<Output = T>,
    {
        Vec2::new(self.right(), self.bottom())
    }
}

#[derive(Debug, Clone)]
struct RectangleRow<T> {
    next_rect: Rectangle<T>,
}

impl<T> Iterator for RectangleRow<T>
where
    T: Copy + AddAssign,
{
    type Item = Rectangle<T>;

    fn next(&mut self) -> Option<Rectangle<T>> {
        let current_rect = self.next_rect;
        self.next_rect.x += self.next_rect.width;
        Some(current_rect)
    }
}

#[derive(Debug, Clone)]
struct RectangleColumn<T> {
    next_rect: Rectangle<T>,
}

impl<T> Iterator for RectangleColumn<T>
where
    T: Copy + AddAssign,
{
    type Item = Rectangle<T>;

    fn next(&mut self) -> Option<Rectangle<T>> {
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
