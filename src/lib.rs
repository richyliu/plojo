mod commands;
mod dispatcher;
mod machine;
mod stroke;
mod translator;

use crate::translator::TextAction;
use machine::raw_stroke::{RawStroke, RawStrokeGeminipr};
use machine::SerialMachine;
use stroke::Stroke;
use translator::{Dictionary, Translation};

pub fn start_georgi() {
    println!("starting plojo...");
    SerialMachine::print_available_ports();

    let dict = mock_dict();
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

                let (command, new_state) = translator::translate(stroke, &dict, translation_state);
                println!("{:?}", command);

                let mut new_controller = controller;
                dispatcher::dispatch(&mut new_controller, command);

                AllState {
                    controller: new_controller,
                    translation_state: new_state,
                }
            },
            AllState {
                controller: dispatcher::new_controller(),
                translation_state: translator::State::default(),
            },
        );
    } else {
        eprintln!("Couldn't find the Georgi port");
    }
}

struct AllState {
    controller: dispatcher::Controller,
    translation_state: translator::State,
}

fn mock_dict() -> Dictionary {
    Dictionary::new(vec![
        (Stroke::new("H-L"), Translation::Text("hello".to_string())),
        (Stroke::new("WORLD"), Translation::Text("world".to_string())),
        (
            Stroke::new("H-L/A"),
            Translation::Text("He..llo".to_string()),
        ),
        (
            Stroke::new("A"),
            Translation::Text("Wrong thing".to_string()),
        ),
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
    ])
}
