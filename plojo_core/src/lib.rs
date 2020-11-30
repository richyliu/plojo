use std::error::Error;
use std::marker::Sized;

mod commands;
mod stroke;

pub use commands::Command;
pub use commands::Key;
pub use stroke::Stroke;

/// Translation from a stroke into a command
pub trait Translator {
    /// Config type
    type C;

    fn new(config: Self::C) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    fn translate(&mut self, stroke: Stroke) -> Vec<Command>;
    fn undo(&mut self) -> Vec<Command>;
}
