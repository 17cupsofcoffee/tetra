// Loosely based on https://github.com/jonhoo/tetris-tutorial

extern crate tetra;

use tetra::error::Result;
use tetra::glm::Vec2;
use tetra::graphics::color;
use tetra::graphics::{self, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

struct Block {
    x: i32,
    y: i32,
    shape: [[bool; 4]; 4],
}

impl Block {
    fn new(x: i32, y: i32) -> Block {
        Block {
            x,
            y,
            shape: [
                [true, false, false, false],
                [true, false, false, false],
                [true, false, false, false],
                [true, false, false, false],
            ],
        }
    }

    fn segments(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.shape.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, exists)| **exists)
                .map(move |(x, _)| (x as i32, y as i32))
        })
    }
}

struct GameState {
    block_texture: Texture,
    block: Block,
    drop_timer: i32,
    move_timer: i32,
    board: [[bool; 10]; 20],
}

impl GameState {
    fn new(ctx: &mut Context) -> Result<GameState> {
        Ok(GameState {
            block_texture: Texture::new(ctx, "./examples/resources/block.png")?,
            block: Block::new(0, -2),
            drop_timer: 0,
            move_timer: 0,
            board: [[false; 10]; 20],
        })
    }

    fn collides(&mut self, move_x: i32, move_y: i32) -> bool {
        for (x, y) in self.block.segments() {
            let board_x = self.block.x + move_x + x as i32;
            let board_y = self.block.y + move_y + y as i32;

            if board_y < 0 {
                continue;
            }

            if board_x < 0
                || board_x > 9
                || board_y > 19
                || self.board[board_y as usize][board_x as usize]
            {
                return true;
            }
        }

        false
    }

    fn lock(&mut self) {
        for (x, y) in self.block.segments() {
            let board_x = self.block.x + x as i32;
            let board_y = self.block.y + y as i32;

            if board_x >= 0 && board_x <= 9 && board_y >= 0 && board_y <= 19 {
                self.board[board_y as usize][board_x as usize] = true;
            }
        }

        self.check_for_clears();
    }

    fn check_for_clears(&mut self) {
        'outer: for y in 0..20 {
            for x in 0..10 {
                if !self.board[y][x] {
                    continue 'outer;
                }
            }

            if y > 0 {
                self.board[y] = self.board[y - 1];
            } else {
                self.board[y] = [false; 10];
            }
        }
    }

    fn board_blocks(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.board.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, exists)| **exists)
                .map(move |(x, _)| (x as i32, y as i32))
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) {
        self.drop_timer += 1;
        self.move_timer += 1;

        if self.drop_timer >= 30 || (self.drop_timer >= 8 && input::is_key_down(ctx, Key::S)) {
            self.drop_timer = 0;

            if self.collides(0, 1) {
                self.lock();
                self.block = Block::new(0, -2);
            } else {
                self.block.y += 1;
            }
        }

        if self.move_timer >= 15 {
            if input::is_key_down(ctx, Key::A) && !self.collides(-1, 0) {
                self.move_timer = 0;
                self.block.x -= 1;
            }

            if input::is_key_down(ctx, Key::D) && !self.collides(1, 0) {
                self.move_timer = 0;
                self.block.x += 1;
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, color::BLACK);

        for (x, y) in self.board_blocks() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(x as f32 * 16.0, y as f32 * 16.0))
                    .color(color::RED),
            );
        }

        for (x, y) in self.block.segments() {
            let board_x = self.block.x + x as i32;
            let board_y = self.block.y + y as i32;

            if board_x >= 0 && board_x <= 9 && board_y >= 0 && board_y <= 19 {
                graphics::draw(
                    ctx,
                    &self.block_texture,
                    DrawParams::new()
                        .position(Vec2::new(board_x as f32 * 16.0, board_y as f32 * 16.0))
                        .color(color::BLUE),
                );
            }
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
