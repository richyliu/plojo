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
                print!("{:?}, ", stroke);

                let (command, new_state) = translate(stroke, &dict, translation_state);
                print!("{:?}, ", command);

                let mut new_controller = controller;
                print!("{:?}, ", new_state);
                let (actions, new_state) = parse_command(new_state, &dict, command);
                println!("{:?}", actions);
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
    Dictionary::new(vec![
        (Stroke::new("H-L"), Translation::Text("Hello".to_string())),
        (Stroke::new("WORLD"), Translation::Text("World".to_string())),
        (
            Stroke::new("H-L/A"),
            Translation::Text("He..llo".to_string()),
        ),
        (
            Stroke::new("A"),
            Translation::Text("Wrong thing".to_string()),
        ),
        (
            Stroke::new("TPHO/WUPB"),
            Translation::Text("no one".to_string()),
        ),
        (
            Stroke::new("KW/A/TP"),
            Translation::Text("request an if".to_string()),
        ),
        (
            Stroke::new("H-L/A/WORLD"),
            Translation::Text("hello a world".to_string()),
        ),
        (
            Stroke::new("KW/H-L/WORLD"),
            Translation::Text("request a hello world".to_string()),
        ),
        (Stroke::new("TPAOD"), Translation::Text("food".to_string())),
        (
            Stroke::new("KPA"),
            Translation::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, true),
            ]),
        ),
        (
            Stroke::new("KPA*"),
            Translation::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, true),
            ]),
        ),
        (
            Stroke::new("-RB"),
            Translation::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, false),
            ]),
        ),
        (
            Stroke::new("S-P"),
            Translation::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, false),
            ]),
        ),
        (
            Stroke::new("*"),
            Translation::Command(Command::Internal(InternalCommand::Undo)),
        ),
        (
            Stroke::new("H*L"),
            Translation::Command(Command::External(ExternalCommand::PrintHello)),
        ),
    ])
}
