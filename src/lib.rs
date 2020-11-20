mod commands;
mod dispatcher;
mod machine;
mod stroke;
mod translator;

pub use commands::{Command, ExternalCommand, InternalCommand};
pub use dispatcher::{controller::Controller, parse_command};
pub use machine::{
    raw_stroke::{RawStroke, RawStrokeGeminipr},
    SerialMachine,
};
pub use stroke::Stroke;
pub use translator::{translate, Dictionary, State, Text, TextAction, Translation};

// #[cfg(test)]
pub fn testing_dict() -> Dictionary {
    // handy helper function for making dictionary entries
    fn row(stroke: &str, translation: &str) -> (Stroke, Vec<Translation>) {
        (
            Stroke::new(stroke),
            vec![Translation::Text(Text::Lit(translation.to_string()))],
        )
    }

    fn row_ta(stroke: &str, text_actions: Vec<TextAction>) -> (Stroke, Vec<Translation>) {
        (
            Stroke::new(stroke),
            vec![Translation::Text(Text::TextAction(text_actions))],
        )
    }

    Dictionary::new(vec![
        (row("H-L", "Hello")),
        (row("WORLD", "World")),
        (row("H-L/A", "He..llo")),
        (row("A", "Wrong thing")),
        (row("TPHO/WUPB", "no one")),
        (row("KW/A/TP", "request an if")),
        (row("H-L/A/WORLD", "hello a world")),
        (row("KW/H-L/WORLD", "request a hello world")),
        (row("PWEUG", "big")),
        (row("PWEUG/PWOEU", "Big Boy")),
        (row("TPAOD", "food")),
        (row_ta(
            "KPA",
            vec![TextAction::space(true, true), TextAction::case(true, true)],
        )),
        (row_ta(
            "KPA*",
            vec![TextAction::space(true, false), TextAction::case(true, true)],
        )),
        (row_ta("-RB", vec![TextAction::space(true, false)])),
        (row_ta("S-P", vec![TextAction::space(true, true)])),
        (
            Stroke::new("*"),
            vec![Translation::Command(Command::Internal(
                InternalCommand::Undo,
            ))],
        ),
        (
            Stroke::new("H*L"),
            vec![Translation::Command(Command::External(
                ExternalCommand::PrintHello,
            ))],
        ),
        (
            Stroke::new("TKAO*ER"),
            vec![
                Translation::Text(Text::Lit("deer and printing hello".to_string())),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
        ),
    ])
}
