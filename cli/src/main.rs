use chrono::prelude::{Local, SecondsFormat};
use clap::{App, Arg, ArgMatches};
use plojo_core::{Controller, Machine, Translator};
use plojo_input_geminipr::{self as geminipr, GeminiprMachine};
use plojo_input_stdin::StdinMachine;
use plojo_output_applescript::ApplescriptController;
use plojo_standard::{Config as StandardTranslatorConfig, StandardTranslator};
use std::{env, path::Path};

pub fn main() {
    let matches = get_arg_matches();

    println!("Starting plojo...");

    /* Print ports */
    if matches.is_present("print-ports") {
        geminipr::print_available_ports();
        return;
    }

    /* Check if only printing output */
    let print_only = matches.is_present("print-only");
    if print_only {
        println!("Only printing output.");
    }

    /* Load machine */
    let is_stdin_machine = matches.is_present("stdin");
    let mut machine: Box<dyn Machine> = if is_stdin_machine {
        Box::new(StdinMachine::new()) as Box<dyn Machine>
    } else {
        let port = match matches.value_of("serial-port") {
            Some(p) => p,
            None => panic!("no serial port provided"),
        };
        let machine = GeminiprMachine::new(port.to_string());
        match machine {
            Ok(m) => Box::new(m),
            Err(e) => panic!("error in loading machine: {}", e),
        }
    };

    /* Load dictionaries */
    println!("Loading dictionaries...");
    // this takes a few seconds
    let raw_dicts = load_dictionaries(matches.value_of("add-dictionary"));
    let config = StandardTranslatorConfig::new().with_raw_dicts(raw_dicts);
    let mut translator = StandardTranslator::new(config).expect("Unable to create translator");
    println!("Loaded dictionaries");

    /* Load controller */
    let mut controller = Box::new(ApplescriptController::new());

    println!("\nReady.\n");

    loop {
        // wait for the next stroke
        let stroke = match machine.read() {
            Ok(s) => s,
            Err(e) => panic!("unable to read stroke: {}", e,),
        };

        // logging time and the stroke
        let mut log = String::new();
        let now = Local::now();
        log.push_str(&format!(
            "{} ",
            now.to_rfc3339_opts(SecondsFormat::Millis, false)
        ));
        log.push_str(&format!("{:?} => ", stroke));

        // translating the stroke
        let commands = if stroke.is_undo() {
            translator.undo()
        } else {
            translator.translate(stroke)
        };
        // logging the command
        log.push_str(&format!("{:?}", commands));

        // performing the command
        if !print_only {
            // delay the dispatching if input is from stdin
            if is_stdin_machine {
                println!("Waiting 1 sec before performing the comand...");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            for command in commands {
                controller.dispatch(command);
            }
        }

        println!("{}", log);
    }
}

/// Get the command line arguments
fn get_arg_matches() -> ArgMatches<'static> {
    App::new("Plojo")
        .version("0.1.0")
        .author("Richard L. <richy.liu.2002@gmail.com>")
        .about("Stenography translator and computer controller")
        .arg(
            Arg::with_name("print-only")
                .short("p")
                .help("Print the commands to stdout instead of dispatching them"),
        )
        .arg(
            Arg::with_name("print-ports")
                .short("t")
                .help("Only print the serial ports that are available"),
        )
        .arg(
            Arg::with_name("stdin")
                .short("i")
                .help("Prompt user for strokes from stdin instead of using the serial port"),
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
                .help("Serial port the machine is connected to"),
        )
        .get_matches()
}

/// Load raw dictionaries from the files with an optional user dictionary
fn load_dictionaries(user_dict: Option<&str>) -> Vec<String> {
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

    if let Some(dict) = user_dict {
        println!("Loading dictionary {} as requested", dict);
        raw_dict_names.push(dict);
    }

    let raw_dicts = raw_dict_names
        .iter()
        .map(|p| path_base.join(p))
        .map(|p| match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(e) => panic!("unable to read dictionary {:?}: {:?}", p, e),
        })
        .collect();

    raw_dicts
}
