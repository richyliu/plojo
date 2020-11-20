#[macro_use]
extern crate lazy_static;

mod commands;
mod dispatcher;
mod load;
mod machine;
mod stroke;
mod translator;

pub use commands::{Command, ExternalCommand, InternalCommand};
pub use dispatcher::{controller::Controller, parse_command};
pub use load::load_dictionary;
pub use machine::{
    raw_stroke::{RawStroke, RawStrokeGeminipr},
    SerialMachine,
};
pub use stroke::Stroke;
pub use translator::{translate, State, Text, TextAction, Translation};

#[cfg(test)]
mod testing_resources;
