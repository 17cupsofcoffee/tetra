// Loosely based on https://github.com/jonhoo/tetris-tutorial

use rand::{self, Rng};
use tetra::glm::Vec2;
use tetra::graphics::color;
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::{self, Context, ContextBuilder, State};

enum BlockShape {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
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
}

impl Block {
    fn new() -> Block {
        let shape = match rand::thread_rng().gen_range(0, 7) {
            0 => BlockShape::I,
            1 => BlockShape::J,
            2 => BlockShape::L,
            3 => BlockShape::O,
            4 => BlockShape::S,
            5 => BlockShape::T,
            _ => BlockShape::Z,
        };

        Block {
            x: 3,
            y: 0,
            shape,
            rotation: BlockRotation::A,
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
            BlockShape::L => match self.rotation {
                BlockRotation::A => &LA,
                BlockRotation::B => &LB,
                BlockRotation::C => &LC,
                BlockRotation::D => &LD,
            },
            BlockShape::O => &O,
            BlockShape::S => match self.rotation {
                BlockRotation::A => &SA,
                BlockRotation::B => &SB,
                BlockRotation::C => &SC,
                BlockRotation::D => &SD,
            },
            BlockShape::T => match self.rotation {
                BlockRotation::A => &TA,
                BlockRotation::B => &TB,
                BlockRotation::C => &TC,
                BlockRotation::D => &TD,
            },
            BlockShape::Z => match self.rotation {
                BlockRotation::A => &ZA,
                BlockRotation::B => &ZB,
                BlockRotation::C => &ZC,
                BlockRotation::D => &ZD,
            },
        }
    }

    fn color(&self) -> Color {
        match self.shape {
            BlockShape::I => Color::rgb(0.0, 1.0, 1.0),
            BlockShape::J => Color::rgb(0.0, 0.0, 1.0),
            BlockShape::L => Color::rgb(1.0, 0.522, 0.106),
            BlockShape::O => Color::rgb(1.0, 0.863, 0.0),
            BlockShape::S => Color::rgb(0.0, 1.0, 0.0),
            BlockShape::T => Color::rgb(0.694, 0.051, 0.788),
            BlockShape::Z => Color::rgb(1.0, 0.0, 0.0),
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

enum Move {
    Left,
    Right,
    RotateCcw,
    RotateCw,
    Drop,
    HardDrop,
}

struct GameState {
    block_texture: Texture,
    block: Block,
    drop_timer: i32,
    move_timer: i32,
    move_queue: Vec<Move>,
    board: [[Option<Color>; 10]; 22],
    score: i32,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            block_texture: Texture::new(ctx, "./examples/resources/block.png")?,
            block: Block::new(),
            drop_timer: 0,
            move_timer: 0,
            move_queue: Vec::new(),
            board: [[None; 10]; 22],
            score: 0,
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
        let color = self.block.color();

        for (x, y) in self.block.segments() {
            if x >= 0 && x <= 9 && y >= 0 && y <= 21 {
                self.board[y as usize][x as usize] = Some(color);
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

            self.score += 1;

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
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.drop_timer += 1;
        self.move_timer += 1;

        if self.drop_timer >= 30 {
            self.drop_timer = 0;
            self.move_queue.push(Move::Drop);
        }

        if input::is_key_pressed(ctx, Key::A)
            || (self.move_timer == 10 && input::is_key_down(ctx, Key::A))
        {
            self.move_timer = 0;
            self.move_queue.push(Move::Left);
        }

        if input::is_key_pressed(ctx, Key::D)
            || (self.move_timer == 10 && input::is_key_down(ctx, Key::D))
        {
            self.move_timer = 0;
            self.move_queue.push(Move::Right);
        }

        if input::is_key_pressed(ctx, Key::Q)
            || (self.move_timer == 10 && input::is_key_down(ctx, Key::Q))
        {
            self.move_timer = 0;
            self.move_queue.push(Move::RotateCcw);
        }

        if input::is_key_pressed(ctx, Key::E)
            || (self.move_timer == 10 && input::is_key_down(ctx, Key::E))
        {
            self.move_timer = 0;
            self.move_queue.push(Move::RotateCw);
        }

        if input::is_key_pressed(ctx, Key::S)
            || (self.move_timer == 10 && input::is_key_down(ctx, Key::S))
        {
            self.move_timer = 0;
            self.drop_timer = 0;
            self.move_queue.push(Move::Drop);
        }

        if input::is_key_pressed(ctx, Key::Space) {
            self.drop_timer = 0;
            self.move_queue.push(Move::HardDrop);
        }

        let next_move = self.move_queue.pop();

        match next_move {
            Some(Move::Left) => {
                if !self.collides(-1, 0) {
                    self.block.x -= 1;
                }
            }
            Some(Move::Right) => {
                if !self.collides(1, 0) {
                    self.block.x += 1;
                }
            }
            Some(Move::RotateCcw) => {
                self.block.rotate_ccw();

                let mut nudge = 0;

                if self.collides(0, 0) {
                    nudge = if self.block.x > 5 { -1 } else { 1 }
                }

                if nudge != 0 && self.collides(nudge, 0) {
                    self.block.rotate_cw();
                } else {
                    self.block.x += nudge;
                }
            }
            Some(Move::RotateCw) => {
                self.block.rotate_cw();

                let mut nudge = 0;

                if self.collides(0, 0) {
                    nudge = if self.block.x > 5 { -1 } else { 1 }
                }

                if nudge != 0 && self.collides(nudge, 0) {
                    self.block.rotate_ccw();
                } else {
                    self.block.x += nudge;
                }
            }
            Some(Move::Drop) => {
                if self.collides(0, 1) {
                    self.lock();
                    self.check_for_clears();

                    if self.check_for_game_over() {
                        println!("Game over! You cleared {} lines.", self.score);
                        tetra::quit(ctx);
                    }

                    self.block = Block::new();
                } else {
                    self.block.y += 1;
                }
            }
            Some(Move::HardDrop) => {
                while !self.collides(0, 1) {
                    self.block.y += 1;
                }

                self.lock();
                self.check_for_clears();

                if self.check_for_game_over() {
                    println!("Game over! You cleared {} lines.", self.score);
                    tetra::quit(ctx);
                }

                self.block = Block::new();
            }
            None => {}
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
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

        let block_color = self.block.color();

        for (x, y) in self.block.segments() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(x as f32 * 16.0, (y - 2) as f32 * 16.0))
                    .color(block_color),
            );
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Tetras")
        .size(10 * 16, 20 * 16)
        .maximized(true)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState::new(ctx)?;

    println!("=== Tetras ===");
    println!("Controls: A and D to move, Q and E to rotate, S to drop one row, Space to hard drop");

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

static LA: [[bool; 4]; 4] = [
    [false, false, true, false],
    [true, true, true, false],
    [false, false, false, false],
    [false, false, false, false],
];

static LB: [[bool; 4]; 4] = [
    [false, true, false, false],
    [false, true, false, false],
    [false, true, true, false],
    [false, false, false, false],
];

static LC: [[bool; 4]; 4] = [
    [false, false, false, false],
    [true, true, true, false],
    [true, false, false, false],
    [false, false, false, false],
];

static LD: [[bool; 4]; 4] = [
    [true, true, false, false],
    [false, true, false, false],
    [false, true, false, false],
    [false, false, false, false],
];

static O: [[bool; 4]; 4] = [
    [false, false, false, false],
    [false, true, true, false],
    [false, true, true, false],
    [false, false, false, false],
];

static SA: [[bool; 4]; 4] = [
    [false, true, true, false],
    [true, true, false, false],
    [false, false, false, false],
    [false, false, false, false],
];

static SB: [[bool; 4]; 4] = [
    [false, true, false, false],
    [false, true, true, false],
    [false, false, true, false],
    [false, false, false, false],
];

static SC: [[bool; 4]; 4] = [
    [false, false, false, false],
    [false, true, true, false],
    [true, true, false, false],
    [false, false, false, false],
];

static SD: [[bool; 4]; 4] = [
    [true, false, false, false],
    [true, true, false, false],
    [false, true, false, false],
    [false, false, false, false],
];

static TA: [[bool; 4]; 4] = [
    [false, true, false, false],
    [true, true, true, false],
    [false, false, false, false],
    [false, false, false, false],
];

static TB: [[bool; 4]; 4] = [
    [false, true, false, false],
    [false, true, true, false],
    [false, true, false, false],
    [false, false, false, false],
];

static TC: [[bool; 4]; 4] = [
    [false, false, false, false],
    [true, true, true, false],
    [false, true, false, false],
    [false, false, false, false],
];

static TD: [[bool; 4]; 4] = [
    [false, true, false, false],
    [true, true, false, false],
    [false, true, false, false],
    [false, false, false, false],
];

static ZA: [[bool; 4]; 4] = [
    [true, true, false, false],
    [false, true, true, false],
    [false, false, false, false],
    [false, false, false, false],
];

static ZB: [[bool; 4]; 4] = [
    [false, false, true, false],
    [false, true, true, false],
    [false, true, false, false],
    [false, false, false, false],
];

static ZC: [[bool; 4]; 4] = [
    [false, false, false, false],
    [true, true, false, false],
    [false, true, true, false],
    [false, false, false, false],
];

static ZD: [[bool; 4]; 4] = [
    [false, true, false, false],
    [true, true, false, false],
    [true, false, false, false],
    [false, false, false, false],
];
