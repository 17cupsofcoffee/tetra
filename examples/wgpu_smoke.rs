//! Temporary smoke test for the wgpu renderer - renders to a canvas and
//! verifies the output pixels via readback.

use tetra::graphics::mesh::{Mesh, ShapeStyle};
use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Canvas, Color, DrawParams, Rectangle, Shader};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

const TINT_SHADER: &str = "
#version 150

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;
uniform vec4 u_diffuse;
uniform vec4 u_tint;

out vec4 o_color;

void main() {
    o_color = texture(u_texture, v_uv) * u_diffuse * v_color * u_tint;
}
";

struct SmokeState {
    frames: usize,
    canvas: Canvas,
    shader: Shader,
    text: Text,
}

impl SmokeState {
    fn new(ctx: &mut Context) -> tetra::Result<SmokeState> {
        Ok(SmokeState {
            frames: 0,
            canvas: Canvas::new(ctx, 64, 64)?,
            shader: Shader::from_fragment_string(ctx, TINT_SHADER)?,
            text: Text::new(
                "wgpu",
                Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 16.0)?,
            ),
        })
    }
}

impl State for SmokeState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        // Off-screen: red square on blue background, in the top-left corner.
        graphics::set_canvas(ctx, &self.canvas);
        graphics::clear(ctx, Color::rgb(0.0, 0.0, 1.0));

        Mesh::rectangle(
            ctx,
            ShapeStyle::Fill,
            Rectangle::new(0.0, 0.0, 16.0, 16.0),
        )?
        .draw(ctx, DrawParams::new().color(Color::rgb(1.0, 0.0, 0.0)));

        graphics::reset_canvas(ctx);

        // On-screen: canvas + custom shader + text.
        graphics::clear(ctx, Color::rgb(0.1, 0.1, 0.1));
        self.canvas.draw(ctx, Vec2::new(8.0, 8.0));

        graphics::set_shader(ctx, &self.shader);
        self.shader.set_uniform(ctx, "u_tint", Color::rgb(0.0, 1.0, 0.0));
        self.canvas.draw(ctx, Vec2::new(96.0, 8.0));
        graphics::reset_shader(ctx);

        self.text.draw(ctx, Vec2::new(8.0, 96.0));

        self.frames += 1;

        if self.frames == 10 {
            let data = self.canvas.get_data(ctx);

            let top_left = data.get_pixel_color(Vec2::new(2, 2));
            let bottom_right = data.get_pixel_color(Vec2::new(60, 60));

            assert!(
                top_left.r > 0.9 && top_left.g < 0.1 && top_left.b < 0.1,
                "expected red at the top-left of the canvas, got {:?}",
                top_left
            );

            assert!(
                bottom_right.b > 0.9 && bottom_right.r < 0.1,
                "expected blue at the bottom-right of the canvas, got {:?}",
                bottom_right
            );

            // MSAA canvas + resolve.
            let msaa_canvas = Canvas::builder(32, 32).samples(4).build(ctx)?;

            graphics::set_canvas(ctx, &msaa_canvas);
            graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));

            Mesh::circle(ctx, ShapeStyle::Fill, Vec2::new(16.0, 16.0), 12.0)?
                .draw(ctx, DrawParams::new().color(Color::rgb(1.0, 1.0, 0.0)));

            graphics::reset_canvas(ctx);

            let msaa_data = msaa_canvas.get_data(ctx);
            let center = msaa_data.get_pixel_color(Vec2::new(16, 16));

            assert!(
                center.r > 0.9 && center.g > 0.9 && center.b < 0.1,
                "expected yellow at the center of the MSAA canvas, got {:?}",
                center
            );

            // Stencil-masked rendering.
            let stencil_canvas = Canvas::builder(32, 32).stencil_buffer(true).build(ctx)?;

            graphics::set_canvas(ctx, &stencil_canvas);
            graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));
            graphics::clear_stencil(ctx, 0);

            // Write a stencil mask covering the left half, without touching
            // the visible pixels.
            graphics::set_color_mask(ctx, false, false, false, false);
            graphics::set_stencil_state(
                ctx,
                graphics::StencilState::write(graphics::StencilAction::Replace, 1),
            );

            Mesh::rectangle(
                ctx,
                ShapeStyle::Fill,
                Rectangle::new(0.0, 0.0, 16.0, 32.0),
            )?
            .draw(ctx, DrawParams::new());

            // Draw white across the whole canvas - only the left half should
            // pass the stencil test.
            graphics::set_color_mask(ctx, true, true, true, true);
            graphics::set_stencil_state(
                ctx,
                graphics::StencilState::read(graphics::StencilTest::EqualTo, 1),
            );

            Mesh::rectangle(
                ctx,
                ShapeStyle::Fill,
                Rectangle::new(0.0, 0.0, 32.0, 32.0),
            )?
            .draw(ctx, DrawParams::new().color(Color::WHITE));

            graphics::set_stencil_state(ctx, graphics::StencilState::disabled());
            graphics::reset_canvas(ctx);

            let stencil_data = stencil_canvas.get_data(ctx);
            let left = stencil_data.get_pixel_color(Vec2::new(4, 16));
            let right = stencil_data.get_pixel_color(Vec2::new(28, 16));

            assert!(
                left.r > 0.9 && left.g > 0.9 && left.b > 0.9,
                "expected white on the stencil-passing side, got {:?}",
                left
            );

            assert!(
                right.r < 0.1 && right.g < 0.1 && right.b < 0.1,
                "expected black on the stencil-masked side, got {:?}",
                right
            );

            // Scissored clear.
            let scissor_canvas = Canvas::new(ctx, 32, 32)?;

            graphics::set_canvas(ctx, &scissor_canvas);
            graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));

            graphics::set_scissor(ctx, Rectangle::new(0, 0, 16, 32));
            graphics::clear(ctx, Color::rgb(0.0, 1.0, 0.0));
            graphics::reset_scissor(ctx);

            graphics::reset_canvas(ctx);

            let scissor_data = scissor_canvas.get_data(ctx);
            let inside = scissor_data.get_pixel_color(Vec2::new(4, 16));
            let outside = scissor_data.get_pixel_color(Vec2::new(28, 16));

            assert!(
                inside.g > 0.9 && inside.r < 0.1,
                "expected green inside the scissor rectangle, got {:?}",
                inside
            );

            assert!(
                outside.g < 0.1,
                "expected black outside the scissor rectangle, got {:?}",
                outside
            );

            println!("SMOKE TEST OK");

            tetra::window::quit(ctx);
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("wgpu smoke test", 320, 240)
        .debug_info(true)
        .build()?
        .run(SmokeState::new)
}
