// Loosely based on https://github.com/jonhoo/tetris-tutorial.
// The scene stack implementation is inspired by Amethyst's state system
// and the ggez-goodies scene stack.

use rand::{self, Rng};
use tetra::audio::Sound;
use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color, DrawParams, Font, Text, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::window;
use tetra::{Context, ContextBuilder, Event, State};

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;
const BLOCK_SIZE: i32 = 16;
const BORDER_SIZE: i32 = 1;
const BOARD_WIDTH: i32 = (10 * BLOCK_SIZE) + BORDER_SIZE;
const BOARD_HEIGHT: i32 = (20 * BLOCK_SIZE) + BORDER_SIZE;
const BOARD_OFFSET_X: i32 = (SCREEN_WIDTH - BOARD_WIDTH) / 2;
const BOARD_OFFSET_Y: i32 = (SCREEN_HEIGHT - BOARD_HEIGHT) / 2;
const SCORE_OFFSET_Y: i32 = BOARD_OFFSET_Y + BOARD_HEIGHT + 4;

fn main() -> tetra::Result {
    ContextBuilder::new("Tetras", 640, 480)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}

// === Scene Management ===

// This trait extends the normal signature of a 'State' with the ability
// to return a transition, effectively making it function like a state
// machine. Later versions of Tetra will probably provide a way to
// do this without defining your own trait!

trait Scene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
}

enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

// Boxing/dynamic dispatch could be avoided here by defining an enum for all
// of your scenes, but that adds a bit of extra boilerplate - your choice!

struct GameState {
    scenes: Vec<Box<dyn Scene>>,
    scaler: ScreenScaler,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let initial_scene = TitleScene::new(ctx)?;

        Ok(GameState {
            scenes: vec![Box::new(initial_scene)],
            scaler: ScreenScaler::with_window_size(
                ctx,
                640,
                480,
                ScalingMode::ShowAllPixelPerfect,
            )?,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());

        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.draw(ctx)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }

        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        graphics::draw(ctx, &self.scaler, Vec2::new(0.0, 0.0));

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }

        Ok(())
    }
}

// === Title Scene ===

struct TitleScene {
    title_text: Text,
    help_text: Text,
}

impl TitleScene {
    fn new(ctx: &mut Context) -> tetra::Result<TitleScene> {
        // Setting a Sound to repeat without holding on to the SoundInstance
        // is usually a bad practice, as it means you can never stop playback.
        // In our case though, we want it to repeat forever, so it's fine!
        Sound::new("./examples/resources/bgm.wav")?.repeat(ctx)?;

        let font = Font::new(ctx, "./examples/resources/DejaVuSansMono.ttf")?;

        Ok(TitleScene {
            title_text: Text::new("Tetras", font, 36.0),
            help_text: Text::new("An extremely legally distinct puzzle game\n\nControls:\nA and D to move\nQ and E to rotate\nS to drop one row\nSpace to hard drop\n\nPress Space to start.", font, 16.0),
        })
    }
}

impl Scene for TitleScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Space) {
            Ok(Transition::Push(Box::new(GameScene::new(ctx)?)))
        } else {
            Ok(Transition::None)
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(ctx, &self.title_text, Vec2::new(16.0, 16.0));
        graphics::draw(ctx, &self.help_text, Vec2::new(16.0, 56.0));

        Ok(Transition::None)
    }
}

// === Game Scene ===

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

struct GameScene {
    backdrop_texture: Texture,
    block_texture: Texture,

    soft_drop_sound: Sound,
    hard_drop_sound: Sound,
    line_clear_sound: Sound,
    game_over_sound: Sound,

    block: Block,
    drop_timer: i32,
    move_timer: i32,
    move_queue: Vec<Move>,
    board: [[Option<Color>; 10]; 22],
    score: i32,
    score_text: Text,
}

impl GameScene {
    fn new(ctx: &mut Context) -> tetra::Result<GameScene> {
        Ok(GameScene {
            backdrop_texture: Texture::new(ctx, "./examples/resources/backdrop.png")?,
            block_texture: Texture::new(ctx, "./examples/resources/block.png")?,

            soft_drop_sound: Sound::new("./examples/resources/softdrop.wav")?,
            hard_drop_sound: Sound::new("./examples/resources/harddrop.wav")?,
            line_clear_sound: Sound::new("./examples/resources/lineclear.wav")?,
            game_over_sound: Sound::new("./examples/resources/gameover.wav")?,

            block: Block::new(),
            drop_timer: 0,
            move_timer: 0,
            move_queue: Vec::new(),
            board: [[None; 10]; 22],
            score: 0,
            score_text: Text::new(
                "Score: 0",
                Font::new(ctx, "./examples/resources/DejaVuSansMono.ttf")?,
                16.0,
            ),
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

    fn check_for_clears(&mut self) -> bool {
        let mut cleared = false;

        'outer: for y in 0..22 {
            for x in 0..10 {
                if self.board[y][x].is_none() {
                    continue 'outer;
                }
            }

            cleared = true;

            self.score += 1;
            self.score_text
                .set_content(format!("Score: {}", self.score));

            for clear_y in (0..=y).rev() {
                if clear_y > 0 {
                    self.board[clear_y] = self.board[clear_y - 1];
                } else {
                    self.board[clear_y] = [None; 10];
                }
            }
        }

        cleared
    }

    fn check_for_game_over(&self) -> bool {
        self.board[0].iter().any(Option::is_some) || self.board[1].iter().any(Option::is_some)
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

impl Scene for GameScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
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
                    self.soft_drop_sound.play_with(ctx, 0.5, 1.0)?;
                    self.lock();

                    if self.check_for_clears() {
                        self.line_clear_sound.play_with(ctx, 0.5, 1.0)?;
                    }

                    if self.check_for_game_over() {
                        self.game_over_sound.play_with(ctx, 0.2, 1.0)?;
                        return Ok(Transition::Pop);
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

                self.hard_drop_sound.play_with(ctx, 0.5, 1.0)?;
                self.lock();

                if self.check_for_clears() {
                    self.line_clear_sound.play_with(ctx, 0.5, 1.0)?;
                }

                if self.check_for_game_over() {
                    self.game_over_sound.play_with(ctx, 0.2, 1.0)?;
                    return Ok(Transition::Pop);
                }

                self.block = Block::new();
            }
            None => {}
        }

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(
            ctx,
            &self.backdrop_texture,
            Vec2::new(BOARD_OFFSET_X as f32, BOARD_OFFSET_Y as f32),
        );

        graphics::draw(
            ctx,
            &self.score_text,
            Vec2::new(BOARD_OFFSET_X as f32, SCORE_OFFSET_Y as f32),
        );

        for (x, y, color) in self.board_blocks() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(
                        (BOARD_OFFSET_X + BORDER_SIZE + x * BLOCK_SIZE) as f32,
                        (BOARD_OFFSET_Y + BORDER_SIZE + (y - 2) * BLOCK_SIZE) as f32,
                    ))
                    .color(color),
            );
        }

        let block_color = self.block.color();

        for (x, y) in self.block.segments() {
            graphics::draw(
                ctx,
                &self.block_texture,
                DrawParams::new()
                    .position(Vec2::new(
                        (BOARD_OFFSET_X + BORDER_SIZE + x * BLOCK_SIZE) as f32,
                        (BOARD_OFFSET_Y + BORDER_SIZE + (y - 2) * BLOCK_SIZE) as f32,
                    ))
                    .color(block_color),
            );
        }

        Ok(Transition::None)
    }
}

// === Static Data ===

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
