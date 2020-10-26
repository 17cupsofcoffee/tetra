use std::error::Error;
use std::fmt::{self, Display, Formatter};

use anyhow::{self, Context as _};
use tetra::{Context, ContextBuilder, State};

#[derive(Debug)]
struct MyCustomError;

impl Display for MyCustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "an error defined by the game, not Tetra")
    }
}

impl Error for MyCustomError {}

fn function_that_will_always_fail() -> Result<(), MyCustomError> {
    Err(MyCustomError)
}

struct GameState;

// The default error type for `State` can be overriden via a type parameter:
impl State<anyhow::Error> for GameState {
    fn update(&mut self, _: &mut Context) -> anyhow::Result<()> {
        function_that_will_always_fail().context("the function failed, surprisingly enough")
    }
}

fn main() -> anyhow::Result<()> {
    // `run` will return whatever error type your `State` implementation uses:
    ContextBuilder::new("Custom Error Handling", 1280, 720)
        .build()?
        .run(|_| Ok(GameState))
}
