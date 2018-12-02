// Loosely based on https://github.com/jonhoo/tetris-tutorial

extern crate rand;
extern crate tetra;

use rand::Rng;
use tetra::error::Result;
use tetra::glm::Vec2;
use tetra::graphics::color;
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

enum BlockShape {
    I,
    J,
}

enum BlockRotation {
    A,
    B,
    C,
    D,
}

struct Block {
    x: i32,
    y: i32,
    shape: BlockShape,
    rotation: BlockRotation,
    color: Color,
}

impl Block {
    fn new() -> Block {
        let mut rng = rand::thread_rng();

        let shape = match rng.gen_range(0, 2) {
            0 => BlockShape::I,
            _ => BlockShape::J,
        };

        Block {
            x: 0,
            y: 0,
            shape,
            rotation: BlockRotation::A,
            color: Color::rgb(rng.gen(), rng.gen(), rng.gen()),
        }
    }

    fn rotate_cw(&mut self) {
        self.rotation = match self.rotation {
            BlockRotation::A => BlockRotation::B,
            BlockRotation::B => BlockRotation::C,
            BlockRotation::C => BlockRotation::D,
            BlockRotation::D => BlockRotation::A,
        }
    }

    fn rotate_ccw(&mut self) {
        self.rotation = match self.rotation {
            BlockRotation::A => BlockRotation::D,
            BlockRotation::B => BlockRotation::A,
            BlockRotation::C => BlockRotation::B,
            BlockRotation::D => BlockRotation::C,
        }
    }

    fn data(&self) -> &'static [[bool; 4]; 4] {
        match self.shape {
            BlockShape::I => match self.rotation {
                BlockRotation::A => &IA,
                BlockRotation::B => &IB,
                BlockRotation::C => &IC,
                BlockRotation::D => &ID,
            },
            BlockShape::J => match self.rotation {
                BlockRotation::A => &JA,
                BlockRotation::B => &JB,
                BlockRotation::C => &JC,
                BlockRotation::D => &JD,
            },
        }
    }

    fn segments(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.data().iter().enumerate().flat_map(move |(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, exists)| **exists)
                .map(move |(x, _)| (x as i32 + self.x, y as i32 + self.y))
        })
    }
}

struct GameState {
    block_texture: Texture,
    block: Block,
    drop_timer: i32,
    move_timer: i32,
    board: [[Option<Color>; 10]; 22],
}

impl GameState {
    fn new(ctx: &mut Context) -> Result<GameState> {
        Ok(GameState {
            block_texture: Texture::new(ctx, "./examples/resources/block.png")?,
            block: Block::new(),
            drop_timer: 0,
            move_timer: 0,
            board: [[None; 10]; 22],
        })
    }

    fn collides(&mut self, move_x: i32, move_y: i32) -> bool {
        for (x, y) in self.block.segments() {
            let new_x = x + move_x;
            let new_y = y + move_y;

            if new_y < 0 {
                continue;
            }

            if new_x < 0
                || new_x > 9
                || new_y > 21
                || self.board[new_y as usize][new_x as usize].is_some()
            {
                return true;
            }
        }

        false
    }

    fn lock(&mut self) {
        for (x, y) in self.block.segments() {
            if x >= 0 && x <= 9 && y >= 0 && y <= 21 {
                self.board[y as usize][x as usize] = Some(self.block.color);
            }
        }
    }

    fn check_for_clears(&mut self) {
        'outer: for y in 0..22 {
            for x in 0..10 {
                if self.board[y][x].is_none() {
                    continue 'outer;
                }
            }

            for clear_y in (0..=y).rev() {
                if clear_y > 0 {
                    self.board[clear_y] = self.board[clear_y - 1];
                } else {
                    self.board[clear_y] = [None; 10];
                }
            }
        }
    }

    fn check_for_game_over(&self) -> bool {
        self.board[0].iter().any(|segment| segment.is_some())
            || self.board[1].iter().any(|segment| segment.is_some())
    }

    fn board_blocks(&self) -> impl Iterator<Item = (i32, i32, Color)> + '_ {
        self.board.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, segment)| segment.is_some())
                .map(move |(x, segment)| (x as i32, y as i32, segment.unwrap()))
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
                self.check_for_clears();

                if self.check_for_game_over() {
                    println!("Game over!");
                    tetra::quit(ctx);
                }

                self.block = Block::new();
            } else {
                self.block.y += 1;
            }
        }

        if self.move_timer >= 15 {
            if input::is_key_down(ctx, Key::Q) {
                self.move_timer = 0;
                self.block.rotate_ccw();

                let mut nudge = 0;

                for (x, _) in self.block.segments() {
                    if x < 0 {
                        nudge = nudge.max(-x);
                    } else if x > 9 {
                        nudge = nudge.min(-x + 9);
                    }
                }

                self.block.x += nudge;
            }

            if input::is_key_down(ctx, Key::E) {
                self.move_timer = 0;
                self.block.rotate_cw();

                let mut nudge = 0;

                for (x, _) in self.block.segments() {
                    if x < 0 {
                        nudge = nudge.max(-x);
                    } else if x > 9 {
                        nudge = nudge.min(-x + 9);
                    }
                }

                self.block.x += nudge;
            }

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

        for (x, y, color) in self.board_blocks() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(x as f32 * 16.0, (y - 2) as f32 * 16.0))
                    .color(color),
            );
        }

        for (x, y) in self.block.segments() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(x as f32 * 16.0, (y - 2) as f32 * 16.0))
                    .color(self.block.color),
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

static IA: [[bool; 4]; 4] = [
    [false, false, false, false],
    [true, true, true, true],
    [false, false, false, false],
    [false, false, false, false],
];

static IB: [[bool; 4]; 4] = [
    [false, false, true, false],
    [false, false, true, false],
    [false, false, true, false],
    [false, false, true, false],
];

static IC: [[bool; 4]; 4] = [
    [false, false, false, false],
    [false, false, false, false],
    [true, true, true, true],
    [false, false, false, false],
];

static ID: [[bool; 4]; 4] = [
    [false, true, false, false],
    [false, true, false, false],
    [false, true, false, false],
    [false, true, false, false],
];

static JA: [[bool; 4]; 4] = [
    [true, false, false, false],
    [true, true, true, false],
    [false, false, false, false],
    [false, false, false, false],
];

static JB: [[bool; 4]; 4] = [
    [false, true, true, false],
    [false, true, false, false],
    [false, true, false, false],
    [false, false, false, false],
];

static JC: [[bool; 4]; 4] = [
    [false, false, false, false],
    [true, true, true, false],
    [false, false, true, false],
    [false, false, false, false],
];

static JD: [[bool; 4]; 4] = [
    [false, true, false, false],
    [false, true, false, false],
    [true, true, false, false],
    [false, false, false, false],
];
