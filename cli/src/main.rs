use clap::{App, Arg, ArgMatches};
use dirs;
use plojo_core::{Command, Translator};
use plojo_input_geminipr as geminipr;
use plojo_standard::StandardTranslator;
use std::{fs, io, path::Path};

mod config;

pub fn main() {
    let matches = get_arg_matches();

    if matches.is_present("print-ports") {
        // only print ports and exit
        println!("[INFO] Only printing available serial ports");
        println!();
        geminipr::print_available_ports();
        println!();
        println!("[INFO] Exiting.");
        return;
    }

    let config_base = matches.value_of("config").map_or_else(
        || Path::new(&dirs::home_dir().unwrap()).join(".plojo"),
        |p: &str| Path::new(p).to_path_buf(),
    );
    let raw_config = fs::read_to_string(config_base.join("config.toml"))
        .expect("unable to read config.toml file");
    let config = config::load(&raw_config).expect("Invalid config format");

    println!("[INFO] Starting plojo...");

    /* Load dictionaries */
    println!("[INFO] Loading dictionaries...");
    let raw_dicts = config.get_dicts(&config_base.join("dicts"));
    let mut translator = StandardTranslator::new(
        raw_dicts,
        vec![],
        config.get_retro_add_space(),
        config.get_space_stroke(),
        config.space_after,
    )
    .expect("unable to create translator");
    println!("[INFO] Loaded dictionaries");

    /* Load machine */
    let mut machine = config.get_input_machine(matches.is_present("stdin"));

    /* Load controller */
    let mut controller = config.get_output_controller(matches.is_present("stdout"));

    println!("[INFO] Ready.");
    println!();

    loop {
        // wait for the next stroke
        let stroke = match machine.read() {
            Ok(s) => s,
            Err(e) => {
                // exit if it is a broken pipe (likely the machine disconnected)
                if let Some(e) = e.downcast_ref::<io::Error>() {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        println!("Machine disconnected. Exiting.");
                        return;
                    }
                }
                panic!("unable to read stroke: {}", e);
            }
        };

        let mut log = String::new();
        log.push_str(&format!("{} ", get_time()));
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
        for command in commands {
            if let Command::TranslatorCommand(cmd) = command {
                translator.handle_command(cmd);
            } else {
                controller.dispatch(command);
            }
        }

        println!("{}", log);
    }
}

fn get_time() -> String {
    use chrono::prelude::{Local, SecondsFormat};
    let now = Local::now();
    now.to_rfc3339_opts(SecondsFormat::Millis, false)
}

/// Get the command line arguments
fn get_arg_matches() -> ArgMatches<'static> {
    App::new("Plojo")
        .version("0.1.0")
        .author("Richard L. <richy.liu.2002@gmail.com>")
        .about("Stenography translator and computer controller")
        .arg(
            Arg::with_name("print-ports")
                .long("ports")
                .help("Only print the serial ports that are available"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("DIR")
                .help("Override location of config files"),
        )
        .arg(
            Arg::with_name("stdin")
                .short("i")
                .help("Overrides the config to use strokes from stdin"),
        )
        .arg(
            Arg::with_name("stdout")
                .short("o")
                .help("Overrides the config and prints to stdout instead of dispatching commands"),
        )
        .get_matches()
}
