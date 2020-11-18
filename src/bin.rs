use plojo_lib as plojo;
use plojo_lib::RawStroke;
use serde_json;
use std::fs;
use std::iter::FromIterator;

pub fn main() {
    println!("starting plojo...");
    plojo::SerialMachine::print_available_ports();

    let dict = load_dict();
    if let Some(port) = plojo::SerialMachine::get_georgi_port() {
        let machine = plojo::SerialMachine::new(port);

        machine.listen(
            |raw,
             AllState {
                 controller,
                 translation_state,
             }| {
                let start = std::time::Instant::now();

                let stroke = plojo::RawStrokeGeminipr::parse_raw(raw).to_stroke();
                print!("{:?} => ", stroke);

                let (command, new_state) = plojo::translate(stroke, &dict, translation_state);
                println!("{:?}", command);

                let mut new_controller = controller;
                let (actions, new_state) = plojo::parse_command(new_state, &dict, command);
                println!("{:?}", start.elapsed());
                new_controller.parse(actions);

                println!("{:?}", start.elapsed());
                AllState {
                    controller: new_controller,
                    translation_state: new_state,
                }
            },
            AllState {
                controller: plojo::Controller::new(),
                translation_state: plojo::State::default(),
            },
        );
    } else {
        eprintln!("Couldn't find the Georgi port");
    }
}

struct AllState {
    controller: plojo::Controller,
    translation_state: plojo::State,
}

fn load_dict() -> plojo::Dictionary {
    let filename = "runtime_files/dict.json";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let value: serde_json::Value =
        serde_json::from_str(&contents).expect("Could not parse the file as JSON");

    // parse the str tuples into a Stroke and a Translation
    let parsed = value
        .as_object()
        .unwrap()
        .into_iter()
        .map(|(stroke_str, translation_str)| {
            (
                plojo::Stroke::new(stroke_str),
                vec![plojo::Translation::Text(
                    translation_str.as_str().unwrap().to_string(),
                )],
            )
        });

    // custom strokes added
    let added = vec![(
        plojo::Stroke::new("*"),
        vec![plojo::Translation::Command(plojo::Command::Internal(
            plojo::InternalCommand::Undo,
        ))],
    )]
    .into_iter();

    plojo::Dictionary::from_iter(parsed.chain(added))
}
