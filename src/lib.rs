mod commands;
mod dispatcher;
mod machine;
mod stroke;
mod translator;

use crate::commands::{ExternalCommand, InternalCommand};
use crate::dispatcher::parse_command;
use crate::translator::translate;
use commands::Command;
use machine::raw_stroke::{RawStroke, RawStrokeGeminipr};
use machine::SerialMachine;
use stroke::Stroke;
use translator::TextAction;
use translator::{Dictionary, Translation};

pub fn start_georgi() {
    println!("starting plojo...");
    SerialMachine::print_available_ports();

    let dict = testing_dict();
    if let Some(port) = SerialMachine::get_georgi_port() {
        let machine = SerialMachine::new(port);

        machine.listen(
            |raw,
             AllState {
                 controller,
                 translation_state,
             }| {
                let stroke = RawStrokeGeminipr::parse_raw(raw).to_stroke();
                print!("{:?} => ", stroke);

                let (command, new_state) = translate(stroke, &dict, translation_state);
                println!("{:?}", command);

                let mut new_controller = controller;
                let (actions, new_state) = parse_command(new_state, &dict, command);
                new_controller.parse(actions);

                AllState {
                    controller: new_controller,
                    translation_state: new_state,
                }
            },
            AllState {
                controller: dispatcher::controller::Controller::new(),
                translation_state: translator::State::default(),
            },
        );
    } else {
        eprintln!("Couldn't find the Georgi port");
    }
}

struct AllState {
    controller: dispatcher::controller::Controller,
    translation_state: translator::State,
}

// #[cfg(test)]
pub fn testing_dict() -> Dictionary {
    // handy helper function for making dictionary entries
    fn row(stroke: &str, translation: &str) -> (Stroke, Vec<Translation>) {
        (
            Stroke::new(stroke),
            vec![Translation::Text(translation.to_string())],
        )
    }

    fn row_ta(stroke: &str, text_actions: Vec<TextAction>) -> (Stroke, Vec<Translation>) {
        (
            Stroke::new(stroke),
            vec![Translation::TextAction(text_actions)],
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
                Translation::Text("deer and printing hello".to_string()),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
        ),
    ])
}
