use plojo_lib as plojo;
use plojo_lib::{load_dictionary, RawStroke};

const DO_OUTPUT: bool = false;

pub fn main() {
    println!("starting plojo...");
    println!("output enable: {:?}", DO_OUTPUT);
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

                let (command, new_state) = plojo::translate(stroke, &dict, translation_state);
                println!("{:?}", command);

                let mut new_controller = controller;
                let (actions, new_state) = plojo::parse_command(new_state, &dict, command);
                if DO_OUTPUT {
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
