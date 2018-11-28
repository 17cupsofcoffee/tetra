extern crate tetra;

use tetra::error::Result;
use tetra::glm::Vec2;
use tetra::graphics::color;
use tetra::graphics::{self, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

enum BlockShape {
    I,
}

enum BlockOrientation {
    A,
    B,
}

struct Block {
    x: usize,
    y: usize,
    shape: BlockShape,
    orientation: BlockOrientation,
}

impl Block {
    fn new(x: usize, y: usize, shape: BlockShape, orientation: BlockOrientation) -> Block {
        Block {
            x,
            y,
            shape,
            orientation,
        }
    }

    fn shape_coords(&self) -> [(usize, usize); 4] {
        match (&self.shape, &self.orientation) {
            (BlockShape::I, BlockOrientation::A) => [(1, 0), (1, 1), (1, 2), (1, 3)],
            (BlockShape::I, BlockOrientation::B) => [(0, 2), (1, 2), (2, 2), (3, 2)],
        }
    }
}

struct GameState {
    block_texture: Texture,
    block: Block,
    drop_timer: i32,
    move_timer: i32,
    board: [[bool; 20]; 10],
}

impl GameState {
    fn new(ctx: &mut Context) -> Result<GameState> {
        Ok(GameState {
            block_texture: Texture::new(ctx, "./examples/resources/block.png")?,
            block: Block::new(0, 0, BlockShape::I, BlockOrientation::A),
            drop_timer: 0,
            move_timer: 0,
            board: [[false; 20]; 10],
        })
    }

    fn can_move(&self, block: &Block, move_x: isize, move_y: isize) -> bool {
        self.block.shape_coords().iter().all(|(seg_x, seg_y)| {
            let dest_x = block.x as isize + *seg_x as isize + move_x;
            let dest_y = block.y as isize + *seg_y as isize + move_y;

            dest_x >= 0
                && dest_x <= 9
                && dest_y >= 0
                && dest_y <= 19
                && !self.board[dest_x as usize][dest_y as usize]
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) {
        self.drop_timer += 1;
        self.move_timer += 1;

        if self.drop_timer >= 30 || (self.drop_timer >= 8 && input::is_key_down(ctx, Key::S)) {
            self.drop_timer = 0;

            if !self.can_move(&self.block, 0, 1) {
                for (x, y) in self.block.shape_coords().iter() {
                    self.board[self.block.x + *x][self.block.y + *y - 1] = true;
                }
                self.block = Block::new(0, 0, BlockShape::I, BlockOrientation::A);
            }
        }

        if self.move_timer >= 15 {
            if input::is_key_down(ctx, Key::A) && self.can_move(&self.block, -1, 0) {
                self.move_timer = 0;
                self.block.x -= 1;
            }

            if input::is_key_down(ctx, Key::D) && self.can_move(&self.block, 11, 0) {
                self.move_timer = 0;
                self.block.x += 1;
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, color::BLACK);

        for (x, column) in self.board.iter().enumerate() {
            for (y, is_filled) in column.iter().enumerate() {
                if *is_filled {
                    graphics::draw(
                        ctx,
                        &self.block_texture,
                        DrawParams::new()
                            .position(Vec2::new(x as f32 * 16.0, y as f32 * 16.0))
                            .color(color::RED),
                    );
                }
            }
        }

        for (x, y) in self.block.shape_coords().iter() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(
                        (self.block.x + *x) as f32 * 16.0,
                        (self.block.y + *y) as f32 * 16.0,
                    )).color(color::BLUE),
            );
        }

        graphics::present(ctx);
    }
}

fn main() -> Result {
    let ctx = &mut ContextBuilder::new()
        .title("Tetras")
        .size(10 * 16, 20 * 16)
        .scale(2)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState::new(ctx)?;

    tetra::run(ctx, state)
}
