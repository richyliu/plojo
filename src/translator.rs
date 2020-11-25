use crate::{Command, Stroke};
use std::error::Error;
use std::marker::Sized;

mod standard;

pub use standard::Config as StandardTranslatorConfig;
pub use standard::StandardTranslator;

/// Translation from a stroke into a command
pub trait Translator {
    type T;

    fn new(config: Self::T) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    fn translate(&mut self, stroke: Stroke) -> Command;
    fn undo(&mut self) -> Command;
}