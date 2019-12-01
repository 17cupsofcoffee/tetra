use crate::graphics;
use crate::math::{Lerp, Mat4, Vec2, Vec3};
use crate::Context;

/// A camera that can be used to transform the scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    /// The position of the camera.
    pub position: Vec2<f32>,

    /// The rotation of the camera, in radians.
    pub rotation: f32,

    /// The zoom level of the camera.
    pub zoom: f32,
}

impl Camera {
    /// Create a new camera.
    pub fn new(position: Vec2<f32>) -> Camera {
        Camera {
            position,
            rotation: 0.0,
            zoom: 1.0,
        }
    }

    /// Creates a new transformation matrix based on the camera's data.
    ///
    /// Pass this to `graphics::set_transform_matrix` to apply the transformation.
    pub fn to_matrix(&self, ctx: &Context) -> Mat4<f32> {
        let mut mat = Mat4::translation_2d(-self.position);

        mat.rotate_z(self.rotation);
        mat.scale_3d(Vec3::new(self.zoom, self.zoom, 1.0));
        mat.translate_2d(Vec2::new(
            graphics::get_canvas_width(ctx) as f32 / 2.0,
            graphics::get_canvas_height(ctx) as f32 / 2.0,
        ));

        mat
    }

    pub fn project(&self, ctx: &Context, point: Vec2<f32>) -> Vec2<f32> {
        self.to_matrix(ctx)
            .inverted()
            .mul_point(Vec3::from_point_2d(point))
            .xy()
    }

    pub fn unproject(&self, ctx: &Context, point: Vec2<f32>) -> Vec2<f32> {
        self.to_matrix(ctx)
            .mul_point(Vec3::from_point_2d(point))
            .xy()
    }

    /// Returns the linear interpolation of two cameras, with `factor` clamped between
    /// 0 and 1.
    ///
    /// See the `vek::Lerp` trait (re-exported from `tetra::math`) for more details.
    pub fn lerp(&self, to: &Camera, factor: f32) -> Camera {
        Lerp::lerp(self, to, factor)
    }

    /// Returns the linear interpolation of two cameras, with `factor` clamped between
    /// 0 and 1. This uses a slower but slightly more precise implementation compared
    /// to `lerp`.
    ///
    /// See the `vek::Lerp` trait (re-exported from `tetra::math`) for more details.
    pub fn lerp_precise(&self, to: &Camera, factor: f32) -> Camera {
        Lerp::lerp_precise(self, to, factor)
    }

    /// Returns the linear interpolation of two cameras, with `factor` unconstrained.
    ///
    /// See the `vek::Lerp` trait (re-exported from `tetra::math`) for more details.
    pub fn lerp_unclamped(&self, to: &Camera, factor: f32) -> Camera {
        Lerp::lerp_unclamped(self, to, factor)
    }

    /// Returns the linear interpolation of two cameras, with `factor` unconstrained.
    /// This uses a slower but slightly more precise implementation compared to
    /// `lerp_unclamped`.
    ///
    /// See the `vek::Lerp` trait (re-exported from `tetra::math`) for more details.
    pub fn lerp_unclamped_precise(&self, to: &Camera, factor: f32) -> Camera {
        Lerp::lerp_unclamped_precise(self, to, factor)
    }
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            position: Vec2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        }
    }
}

impl Lerp<f32> for Camera {
    type Output = Camera;

    fn lerp_unclamped(from: Camera, to: Camera, factor: f32) -> Camera {
        Camera {
            position: Lerp::lerp_unclamped(from.position, to.position, factor),
            rotation: Lerp::lerp_unclamped(from.rotation, to.rotation, factor),
            zoom: Lerp::lerp_unclamped(from.zoom, to.zoom, factor),
        }
    }

    fn lerp_unclamped_precise(from: Camera, to: Camera, factor: f32) -> Camera {
        Camera {
            position: Lerp::lerp_unclamped_precise(from.position, to.position, factor),
            rotation: Lerp::lerp_unclamped_precise(from.rotation, to.rotation, factor),
            zoom: Lerp::lerp_unclamped_precise(from.zoom, to.zoom, factor),
        }
    }
}

impl<'a> Lerp<f32> for &'a Camera {
    type Output = Camera;

    fn lerp_unclamped(from: &'a Camera, to: &'a Camera, factor: f32) -> Camera {
        Camera {
            position: Lerp::lerp_unclamped(from.position, to.position, factor),
            rotation: Lerp::lerp_unclamped(from.rotation, to.rotation, factor),
            zoom: Lerp::lerp_unclamped(from.zoom, to.zoom, factor),
        }
    }

    fn lerp_unclamped_precise(from: &'a Camera, to: &'a Camera, factor: f32) -> Camera {
        Camera {
            position: Lerp::lerp_unclamped_precise(from.position, to.position, factor),
            rotation: Lerp::lerp_unclamped_precise(from.rotation, to.rotation, factor),
            zoom: Lerp::lerp_unclamped_precise(from.zoom, to.zoom, factor),
        }
    }
}
