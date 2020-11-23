use plojo_lib as plojo;
use plojo_lib::{RawStroke, Translator};
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

    let raw_dict =
        std::fs::read_to_string("runtime_files/dict.json").expect("Unable to load the dictionary");
    let initial_translator =
        plojo::StandardTranslator::new(plojo::StandardTranslatorConfig::new(raw_dict, vec![]))
            .expect("Unable to create translator");

    if let Some(port) = plojo::SerialMachine::get_georgi_port() {
        let machine = plojo::SerialMachine::new(port);

        machine.listen(
            |raw,
             AllState {
                 controller,
                 translator,
             }| {
                let stroke = plojo::RawStrokeGeminipr::parse_raw(raw).to_stroke();
                print!("{:?} => ", stroke);

                let mut new_translator = translator;
                let command = if stroke.is_undo() {
                    new_translator.undo()
                } else {
                    new_translator.translate(stroke)
                };
                println!("{:?}", command);

                let mut new_controller = controller;
                let actions = plojo::parse_command(command);
                if do_output {
                    new_controller.dispatch(actions);
                }

                AllState {
                    controller: new_controller,
                    translator: new_translator,
                }
            },
            AllState {
                controller: plojo::Controller::new(),
                translator: initial_translator,
            },
        );
    } else {
        eprintln!("Couldn't find the Georgi port");
    }
}

struct AllState {
    controller: plojo::Controller,
    translator: plojo::StandardTranslator,
}
