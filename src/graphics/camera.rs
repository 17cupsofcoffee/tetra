use super::Rectangle;
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
    /// Creates a new camera with the given viewport size.
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

    /// Creates a new camera, with the viewport size set to match the size of the window.
    ///
    /// Note that if the window is resized, the camera's viewport size will *not* automatically
    /// update. If you need to keep the window size and the viewport size in sync, then call
    /// `set_viewport_size` in your `State`'s `event` method when `Event::Resized` is fired.
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
    /// your scene. To disable the transformation, call `graphics::reset_transform_matrix`.
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
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx))`.
    pub fn mouse_position(&self, ctx: &Context) -> Vec2<f32> {
        self.project(input::get_mouse_position(ctx))
    }

    /// Returns the X co-ordinate of the mouse's position in camera space.
    ///
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx)).x`.
    pub fn mouse_x(&self, ctx: &Context) -> f32 {
        self.mouse_position(ctx).x
    }

    /// Returns the Y co-ordinate of the mouse's position in camera space.
    ///
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx)).y`.
    pub fn mouse_y(&self, ctx: &Context) -> f32 {
        self.mouse_position(ctx).y
    }

    /// Returns the visible rectangle. Everything inside of this rectangle is drawn on the screen.
    pub fn visible_rect(&self) -> Rectangle {
        let viewport_width = self.viewport_width / self.zoom;
        let viewport_height = self.viewport_height / self.zoom;
        let left = self.position.x - viewport_width / 2.0;
        let top = self.position.y - viewport_height / 2.0;

        Rectangle {
            x: left,
            y: top,
            width: viewport_width,
            height: viewport_height,
        }
    }
}

#[test]
fn validate_camera_visible_rect() {
    let mut camera = Camera::new(800., 600.);
    // Camera is centered on 0.0 / 0.0 by default
    assert_eq!(
        camera.visible_rect(),
        Rectangle {
            x: -400.,
            y: -300.,
            width: 800.,
            height: 600.
        }
    );

    // Zooming in will reduce the visible rect size and x/y position by half
    camera.zoom = 2.0;
    assert_eq!(
        camera.visible_rect(),
        Rectangle {
            x: -200.,
            y: -150.,
            width: 400.,
            height: 300.
        }
    );

    // Moving the camera will simply move the x/y position
    camera.position = Vec2::new(-100., 100.);
    assert_eq!(
        camera.visible_rect(),
        Rectangle {
            x: -300.,
            y: -50.,
            width: 400.,
            height: 300.
        }
    );
}
