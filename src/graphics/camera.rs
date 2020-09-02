use super::Rectangle;
use crate::input;
use crate::math::{Mat4, Vec2, Vec3};
use crate::window;
use crate::Context;

/// A camera that can be used to transform the player's view of the scene.
///
/// To apply the transformation, call the `as_matrix` method and pass the
/// resulting `Mat4` to [`graphics::set_transform_matrix`](../fn.set_transform_matrix.html).
/// To disable it, call [`graphics::reset_transform_matrix`](../fn.set_transform_matrix.html).
///
/// The camera's matrix is cached internally as an optimization. After adjusting parameters
/// on the camera, you can call the `update` method to recalculate the matrix.
///
/// # Examples
///
/// The [`camera`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/camera.rs)
/// example demonstrates how a camera can be used to transform a simple
/// scene.
#[derive(Debug, Clone)]
pub struct Camera {
    /// The position of the camera.
    ///
    /// Note that this defines the center point of the view, rather than the top-left.
    /// This makes it easy to position the camera relative to your game objects - for
    /// example, to focus the camera on the player, you can just set the camera
    /// position to match the player's position.
    ///
    /// You may need to take this behaviour into account when positioning the camera,
    /// however. For example, if the viewport width or height is an odd number, setting
    /// the position to a whole number will mean that the view will not be aligned with
    /// the pixel grid, which may cause issues for pixel-perfect rendering.
    pub position: Vec2<f32>,

    /// The rotation of the camera, in radians.
    pub rotation: f32,

    /// The zoom level of the camera.
    ///
    /// This is expressed as a scale factor - `0.5` will shrink everything by half,
    /// while `2.0` will make everything twice as big.
    pub zoom: f32,

    /// The width of the camera's viewport.
    ///
    /// This is primarily used for calculating where the center of the screen is,
    /// and usually should match the size of the target you're currently rendering to
    /// (e.g. the screen, or a `Canvas`).
    pub viewport_width: f32,

    /// The height of the camera's viewport.
    ///
    /// This is primarily used for calculating where the center of the screen is,
    /// and usually should match the size of the target you're currently rendering to
    /// (e.g. the screen, or a `Canvas`).
    pub viewport_height: f32,

    matrix: Mat4<f32>,
}

impl Camera {
    /// Creates a new camera with the given viewport size.
    ///
    /// The provided size usually should match the size of the target you're currently rendering to
    /// (e.g. the screen, or a `Canvas`).
    pub fn new(viewport_width: f32, viewport_height: f32) -> Camera {
        Camera {
            position: Vec2::zero(),
            rotation: 0.0,
            zoom: 1.0,
            viewport_width,
            viewport_height,

            matrix: Mat4::translation_2d(Vec2::new(viewport_width / 2.0, viewport_height / 2.0)),
        }
    }

    /// Creates a new camera, with the viewport size set to match the size of the window.
    ///
    /// This is a useful shortcut if your game renders at a 1:1 ratio with the game window.
    /// If you're rendering to a differently sized target (e.g. a `Canvas` or a
    /// `ScreenScaler`), then you should use call `new` with the target size
    /// instead.
    ///
    /// Note that if the window is resized, the camera's viewport size will *not* automatically
    /// update. If you need to keep the window size and the viewport size in sync, then call
    /// `set_viewport_size` in your `State`'s `event` method when `Event::Resized` is fired.
    pub fn with_window_size(ctx: &Context) -> Camera {
        let (width, height) = window::get_size(ctx);
        Camera::new(width as f32, height as f32)
    }

    /// Sets the size of the camera's viewport.
    ///
    /// The provided size usually should match the size of the target you're currently rendering to
    /// (e.g. the screen, or a `Canvas`).
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Recalculates the transformation matrix, based on the data currently contained
    /// within the camera.
    pub fn update(&mut self) {
        self.matrix = Mat4::translation_2d(-self.position);
        self.matrix.rotate_z(self.rotation);
        self.matrix.scale_3d(Vec3::new(self.zoom, self.zoom, 1.0));
        self.matrix.translate_2d(Vec2::new(
            self.viewport_width / 2.0,
            self.viewport_height / 2.0,
        ));
    }

    /// Returns the current transformation matrix.
    ///
    /// Pass this to `graphics::set_transform_matrix` to apply the transformation to
    /// your scene. To disable the transformation, call `graphics::reset_transform_matrix`.
    ///
    /// The matrix is cached internally, so calling this method multiple times will not
    /// cause it to be recalculated from scratch.
    pub fn as_matrix(&self) -> Mat4<f32> {
        self.matrix
    }

    /// Projects a point from world co-ordinates to camera co-ordinates.
    pub fn project(&self, point: Vec2<f32>) -> Vec2<f32> {
        let mut proj = Vec2::new(
            (point.x - self.viewport_width / 2.0) / self.zoom,
            (point.y - self.viewport_height / 2.0) / self.zoom,
        );

        proj.rotate_z(-self.rotation);
        proj += self.position;

        proj
    }

    /// Projects a point from camera co-ordinates to world co-ordinates.
    pub fn unproject(&self, point: Vec2<f32>) -> Vec2<f32> {
        let mut unproj = point - self.position;
        unproj.rotate_z(self.rotation);

        unproj.x = unproj.x * self.zoom + self.viewport_width / 2.0;
        unproj.y = unproj.y * self.zoom + self.viewport_height / 2.0;

        unproj
    }

    /// Returns the mouse's position in camera co-ordinates.
    ///
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx))`.
    /// As such, it does not take into account any other transformations
    /// being made to the view (e.g. screen scaling).
    pub fn mouse_position(&self, ctx: &Context) -> Vec2<f32> {
        self.project(input::get_mouse_position(ctx))
    }

    /// Returns the X co-ordinate of the mouse's position in camera co-ordinates.
    ///
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx)).x`.
    /// As such, it does not take into account any other transformations
    /// being made to the view (e.g. screen scaling).
    pub fn mouse_x(&self, ctx: &Context) -> f32 {
        self.mouse_position(ctx).x
    }

    /// Returns the Y co-ordinate of the mouse's position in camera co-ordinates.
    ///
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx)).y`.
    /// As such, it does not take into account any other transformations
    /// being made to the view (e.g. screen scaling).
    pub fn mouse_y(&self, ctx: &Context) -> f32 {
        self.mouse_position(ctx).y
    }

    /// Calculates the visible rectangle of the camera.
    ///
    /// When used on a rotated camera, this will return the smallest rectangle that
    /// contains the full camera viewport.
    ///
    /// Note that this method does not take into account any other transformations being
    /// made to the view (e.g. screen scaling).
    pub fn visible_rect(&self) -> Rectangle {
        let viewport_width = self.viewport_width / self.zoom;
        let viewport_height = self.viewport_height / self.zoom;

        let half_viewport_width = viewport_width / 2.0;
        let half_viewport_height = viewport_height / 2.0;

        if self.rotation.abs() > f32::EPSILON {
            // Rotate the top-left and bottom-left point, then get the max x and y from both vectors.
            // This is the range of the bounding box that contains this rectangle.
            let mut top_left = Vec2::new(-half_viewport_width, -half_viewport_height);
            let mut bottom_left = Vec2::new(-half_viewport_width, half_viewport_height);

            top_left.rotate_z(self.rotation);
            bottom_left.rotate_z(self.rotation);

            let largest_x = f32::max(top_left.x.abs(), bottom_left.x.abs());
            let largest_y = f32::max(top_left.y.abs(), bottom_left.y.abs());

            let left = self.position.x - largest_x;
            let top = self.position.y - largest_y;

            // The largest x and y are the distance from the center, so the width is twice that.
            let width = largest_x * 2.0;
            let height = largest_y * 2.0;

            Rectangle {
                x: left,
                y: top,
                width,
                height,
            }
        } else {
            // Quick happy path with no rotation
            let left = self.position.x - half_viewport_width;
            let top = self.position.y - half_viewport_height;

            Rectangle {
                x: left,
                y: top,
                width: viewport_width,
                height: viewport_height,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_projections() {
        let mut camera = Camera::new(128.0, 256.0);

        let proj_initial = camera.project(Vec2::zero());
        let unproj_initial = camera.unproject(proj_initial);

        assert_eq!(proj_initial, Vec2::new(-64.0, -128.0));
        assert_eq!(unproj_initial, Vec2::zero());

        camera.position = Vec2::new(16.0, 16.0);

        let proj_positioned = camera.project(Vec2::zero());
        let unproj_positioned = camera.unproject(proj_positioned);

        assert_eq!(proj_positioned, Vec2::new(-48.0, -112.0));
        assert_eq!(unproj_positioned, Vec2::zero());

        camera.zoom = 2.0;

        let proj_zoomed = camera.project(Vec2::zero());
        let unproj_zoomed = camera.unproject(proj_zoomed);

        assert_eq!(proj_zoomed, Vec2::new(-16.0, -48.0));
        assert_eq!(unproj_zoomed, Vec2::zero());

        camera.rotation = std::f32::consts::FRAC_PI_2;

        let proj_rotated = camera.project(Vec2::zero());
        let unproj_rotated = camera.unproject(proj_rotated);

        assert!(proj_rotated.x + 48.0 <= 0.001);
        assert!(proj_rotated.y - 48.0 <= 0.001);
        assert!(unproj_rotated.x.abs() <= 0.001);
        assert!(unproj_rotated.y.abs() <= 0.001);
    }

    #[test]
    fn validate_camera_visible_rect() {
        let mut camera = Camera::new(800.0, 600.0);

        // Camera is centered on 0.0 / 0.0 by default
        assert_eq!(
            camera.visible_rect(),
            Rectangle {
                x: -400.0,
                y: -300.0,
                width: 800.0,
                height: 600.0
            }
        );

        // Zooming in will reduce the visible rect size and x/y position by half
        camera.zoom = 2.0;

        assert_eq!(
            camera.visible_rect(),
            Rectangle {
                x: -200.0,
                y: -150.0,
                width: 400.0,
                height: 300.0
            }
        );

        // Moving the camera will simply move the x/y position
        camera.position = Vec2::new(-100.0, 100.0);

        assert_eq!(
            camera.visible_rect(),
            Rectangle {
                x: -300.0,
                y: -50.0,
                width: 400.0,
                height: 300.0
            }
        );

        // Rotating the camera by 0.5 * pi will rotate the rectangle by 90 degrees,
        // so the width and height will be swapped
        camera.rotation = std::f32::consts::FRAC_PI_2;

        // We need to manually compare this to a small value because of rounding errors
        let rect = camera.visible_rect();
        assert!(rect.x + 250.0 < 0.001);
        assert!(rect.y + 100.0 < 0.001);
        assert!(rect.width - 300.0 < 0.001);
        assert!(rect.height - 400.0 < 0.001);
    }
}
