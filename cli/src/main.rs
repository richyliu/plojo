use chrono::prelude::{Local, SecondsFormat};
use plojo_core::{Controller, Machine, Translator};
use plojo_input_geminipr as geminipr;
use plojo_input_geminipr::GeminiprMachine;
use plojo_output_applescript::ApplescriptController;
use plojo_standard::{Config as StandardTranslatorConfig, StandardTranslator};

use clap::{App, Arg};

use std::env;
use std::path::Path;

pub fn main() {
    let matches = App::new("Plojo")
        .version("0.1.0")
        .author("Richard L. <richy.liu.2002@gmail.com>")
        .about("Stenography translator and computer controller")
        .arg(
            Arg::with_name("print-only")
                .short("p")
                .help("Print the commands to stdout instead of dispatch them"),
        )
        .arg(
            Arg::with_name("print-ports")
                .short("t")
                .help("Print the serial ports that are available then exit"),
        )
        .arg(
            Arg::with_name("add-dictionary")
                .short("a")
                .takes_value(true)
                .help("Add an user dictionary"),
        )
        .arg(
            Arg::with_name("serial-port")
                .short("s")
                .takes_value(true)
                .required_unless("print-ports")
                .help("Serial port the machine is connected to"),
        )
        .get_matches();

    if matches.is_present("print-ports") {
        geminipr::print_available_ports();
        return;
    }

    let print_only = matches.is_present("print-only");
    if print_only {
        println!("Only printing output.");
    }

    println!("Starting plojo...");

    println!("Loading dictionaries...");

    let path_base = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime_files");
    let mut raw_dict_names = vec![
        "dict.json",
        "fingerspelling.json",
        "fingerspelling-RBGS.json",
        "numbers.json",
        "thumb_numbers.json",
        "nav.json",
        "modifiers-single-stroke.json",
    ];

    if let Some(dict) = matches.value_of("add-dictionary") {
        println!("Loading dictionary {} as requested", dict);
        raw_dict_names.push(dict);
    }

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

    // unwrap is safe here because serial-port is required
    let port = matches.value_of("serial-port").unwrap();
    let machine = GeminiprMachine::new(port.to_string());

    println!("\nReady.\n");

    struct State {
        controller: Box<dyn Controller>,
        translator: StandardTranslator,
    }

    machine.listen(
        |stroke, state| {
            let mut log = String::new();
            let now = Local::now();
            log.push_str(&format!(
                "{} ",
                now.to_rfc3339_opts(SecondsFormat::Millis, false)
            ));

            log.push_str(&format!("{:?} => ", stroke));

            let commands = if stroke.is_undo() {
                state.translator.undo()
            } else {
                state.translator.translate(stroke)
            };
            log.push_str(&format!("{:?}", commands));

            if !print_only {
                for command in commands {
                    state.controller.dispatch(command);
                }
            }

            println!("{}", log);
        },
        &mut State {
            controller: Box::new(ApplescriptController::new()),
            translator: initial_translator,
        },
    );
}
