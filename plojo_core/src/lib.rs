use std::{error::Error, marker::Sized};

mod commands;
mod stroke;

pub use commands::Command;
pub use commands::Key;
pub use commands::Modifier;
pub use commands::SpecialKey;
pub use stroke::Stroke;

/// Translation from a stroke into a command
pub trait Translator {
    fn translate(&mut self, stroke: Stroke) -> Vec<Command>;
    fn undo(&mut self) -> Vec<Command>;
}

/// Controller that can perform a command
pub trait Controller {
    fn new() -> Self
    where
        Self: Sized;
    fn dispatch(&mut self, command: Command);
}

/// A stenography machine (or equivalent)
pub trait Machine {
    /// Waits until a new stroke is read
    fn read(&mut self) -> Result<Stroke, Box<dyn Error>>;
}
