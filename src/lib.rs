mod commands;
mod dispatcher;
mod machine;
mod stroke;
mod translator;

use machine::raw_stroke::{RawStroke, RawStrokeGeminipr};
use machine::SerialMachine;
use stroke::Stroke;
use translator::{Dictionary, Translation};

pub fn start_georgi() {
    println!("starting plojo...");
    SerialMachine::print_available_ports();

    let dict = Dictionary::new(vec![
        (Stroke::new("H-L"), Translation::text("Hello")),
        (Stroke::new("WORLD"), Translation::text("World")),
        (Stroke::new("H-L/A"), Translation::text("He..llo")),
        (Stroke::new("A"), Translation::text("Wrong thing")),
    ]);
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
