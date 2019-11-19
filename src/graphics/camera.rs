use crate::math::{Mat4, Vec2, Vec3};
use crate::graphics;
use crate::Context;

pub struct Camera {
    pub position: Vec2<f32>,
    pub rotation: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new(position: Vec2<f32>, rotation: f32, zoom: f32) -> Camera {
        Camera {
            position,
            rotation,
            zoom,
        }
    }

    pub fn to_matrix(&self, ctx: &Context) -> Mat4<f32> {
        let mut mat = Mat4::translation_2d(-self.position);

        mat.rotate_z(self.rotation);
        mat.scale_3d(Vec3::new(self.zoom, self.zoom, 1.0));
        mat.translate_2d(Vec2::new(
            graphics::get_viewport_width(ctx) as f32 / 2.0,
            graphics::get_viewport_height(ctx) as f32 / 2.0,
        ));

        mat
    }
}
