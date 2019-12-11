use crate::input;
use crate::math::{Mat4, Vec2, Vec3};
use crate::window;
use crate::Context;

/// A camera that can be used to transform the scene.
#[derive(Debug, Clone)]
pub struct Camera {
    /// The position of the camera.
    pub position: Vec2<f32>,

    /// The rotation of the camera, in radians.
    pub rotation: f32,

    /// The zoom level of the camera.
    pub zoom: f32,

    /// The width of the camera's viewport.
    pub viewport_width: f32,

    /// The height of the camera's viewport.
    pub viewport_height: f32,

    matrix: Mat4<f32>,
}

impl Camera {
    /// Creates a new camera.
    pub fn new(viewport_width: f32, viewport_height: f32) -> Camera {
        Camera {
            position: Vec2::zero(),
            rotation: 0.0,
            zoom: 1.0,
            viewport_width,
            viewport_height,

            matrix: Mat4::identity(),
        }
    }

    /// Creates a new camera, with the viewport size set to match the window.
    ///
    /// If the window is resizable, make sure that you call `set_viewport_size` when
    /// the size changes!
    pub fn with_window_size(ctx: &Context) -> Camera {
        let (width, height) = window::get_size(ctx);
        Camera::new(width as f32, height as f32)
    }

    /// Sets the size of the camera's viewport.
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
    /// your scene!
    pub fn as_matrix(&self) -> Mat4<f32> {
        self.matrix
    }

    /// Projects a point from screen co-ordinates to camera co-ordinates.
    pub fn project(&self, point: Vec2<f32>) -> Vec2<f32> {
        self.as_matrix()
            .inverted()
            .mul_point(Vec3::from_point_2d(point))
            .xy()
    }

    /// Projects a point from camera co-ordinates to screen co-ordinates.
    pub fn unproject(&self, point: Vec2<f32>) -> Vec2<f32> {
        self.as_matrix().mul_point(Vec3::from_point_2d(point)).xy()
    }

    /// Returns the mouse's position in camera space.
    ///
    /// This is a shortcut for calling `camera.project(input::get_mouse_position(ctx))`.
    pub fn mouse_position(&self, ctx: &Context) -> Vec2<f32> {
        self.project(input::get_mouse_position(ctx))
    }

    /// Returns the X co-ordinate of the mouse's position in camera space.
    ///
    /// This is a shortcut for calling `camera.project(input::get_mouse_position(ctx)).x`.
    pub fn mouse_x(&self, ctx: &Context) -> f32 {
        self.mouse_position(ctx).x
    }

    /// Returns the Y co-ordinate of the mouse's position in camera space.
    ///
    /// This is a shortcut for calling `camera.project(input::get_mouse_position(ctx)).y`.
    pub fn mouse_y(&self, ctx: &Context) -> f32 {
        self.mouse_position(ctx).y
    }
}
