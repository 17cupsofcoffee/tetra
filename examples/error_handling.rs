use std::error::Error;
use std::fmt::{self, Display, Formatter};

use anyhow::{self, Context as _};
use tetra::{Context, ContextBuilder, StateWithError};

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

impl StateWithError for GameState {
    type Error = anyhow::Error;

    fn update(&mut self, _: &mut Context) -> anyhow::Result<()> {
        function_that_will_always_fail().context("the function failed, surprisingly enough")
    }
}

fn main() -> anyhow::Result<()> {
    ContextBuilder::new("Custom Error Handling", 1280, 720)
        .build()?
        .run(|_| Ok(GameState))
}
