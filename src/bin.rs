use plojo_lib as plojo;
use plojo_lib::{load_dictionary, RawStroke};
use std::env;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let do_output = args.len() == 2;
    if do_output {
        println!("You have passed in an argument, so output is ENABLED");
    } else {
        println!("You have not passed in any arguments, so output is DISABLED");
    }

    println!("\nStarting plojo...");
    plojo::SerialMachine::print_available_ports();

    let dict = load_dictionary("runtime_files/dict.json").expect("unable to load dictionary");
    if let Some(port) = plojo::SerialMachine::get_georgi_port() {
        let machine = plojo::SerialMachine::new(port);

        machine.listen(
            |raw,
             AllState {
                 controller,
                 translation_state,
             }| {
                let stroke = plojo::RawStrokeGeminipr::parse_raw(raw).to_stroke();
                print!("{:?} => ", stroke);

                let (command, new_state) = if stroke.is_undo() {
                    plojo::undo(&dict, translation_state)
                } else {
                    plojo::translate(stroke, &dict, translation_state)
                };
                println!("{:?}", command);

                let mut new_controller = controller;
                let actions = plojo::parse_command(command);
                if do_output {
                    new_controller.dispatch(actions);
                }

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
