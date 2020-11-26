use crate::{Command, Stroke};
use std::error::Error;
use std::marker::Sized;

/// Translation from a stroke into a command
pub trait Translator {
    /// Config type
    type C;

    fn new(config: Self::C) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    fn translate(&mut self, stroke: Stroke) -> Command;
    fn undo(&mut self) -> Command;
}
