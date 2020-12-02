use chrono::prelude::{Local, SecondsFormat};
use plojo_core::Translator;
use plojo_input::{RawStroke, RawStrokeGeminipr, SerialMachine};
use plojo_standard::{Config as StandardTranslatorConfig, StandardTranslator};

use std::env;
use std::path::Path;

mod controller;

use controller::{ApplescriptController, Controller};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let do_output = args.len() == 2;
    if do_output {
        println!("You have passed in an argument, so output is ENABLED");
    } else {
        println!("You have not passed in any arguments, so output is DISABLED");
    }

    println!("Starting plojo...");
    SerialMachine::print_available_ports();

    println!("Loading dictionaries...");
    let path_base = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime_files");
    let raw_dict_names = [
        "dict.json",
        "fingerspelling.json",
        "fingerspelling-RBGS.json",
        "numbers.json",
        "thumb_numbers.json",
        "nav.json",
        "modifiers-single-stroke.json",
        args.get(1).map_or("empty.json", |s| &s),
    ];
    let raw_dicts = raw_dict_names
        .iter()
        .map(|p| path_base.join(p))
        .map(|p| match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(e) => panic!("Unable to read dictionary {:?}: {:?}", p, e),
        })
        .collect();
    let config = StandardTranslatorConfig::new().with_raw_dicts(raw_dicts);
    let initial_translator = StandardTranslator::new(config).expect("Unable to create translator");
    println!("Loaded dictionaries: {:?}", raw_dict_names);

    if let Some(port) = SerialMachine::get_georgi_port() {
        let machine = SerialMachine::new(port);

        struct State {
            controller: Box<dyn Controller>,
            translator: StandardTranslator,
        }

        machine.listen(
            |raw, state| {
                let now = Local::now();
                print!("{} ", now.to_rfc3339_opts(SecondsFormat::Millis, false),);

                let stroke = RawStrokeGeminipr::parse_raw(raw).to_stroke();
                print!("{:?} => ", stroke);

                let commands = if stroke.is_undo() {
                    state.translator.undo()
                } else {
                    state.translator.translate(stroke)
                };
                println!("{:?}", commands);

                if do_output {
                    for command in commands {
                        state.controller.dispatch(command);
                    }
                }
            },
            &mut State {
                controller: Box::new(ApplescriptController::new()),
                translator: initial_translator,
            },
        );
    } else {
        eprintln!("Couldn't find the Georgi port");
    }
}