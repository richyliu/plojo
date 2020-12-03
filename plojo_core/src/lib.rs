use std::{any::Any, error::Error, marker::Sized};

mod commands;
mod stroke;

pub use commands::Command;
pub use commands::Key;
pub use commands::Modifier;
pub use commands::SpecialKey;
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

/// Controller that can perform a command
pub trait Controller {
    fn new() -> Self
    where
        Self: Sized;
    fn dispatch(&mut self, command: Command);
}

/// A stenography machine (or equivalent)
pub trait Machine {
    /// Config type
    type C;

    fn new(config: Self::C) -> Self
    where
        Self: Sized;
    /// Respond to a new stroke being pressed
    fn listen<T, U>(&self, on_stroke: T, state: &mut U)
    where
        T: Fn(Stroke, &mut U),
        U: Any;
}
