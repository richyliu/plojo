use plojo_lib as plojo;
use plojo_lib::{RawStroke, Text};
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
                println!("{:?}", stroke);

                let (command, new_state) = plojo::translate(stroke, &dict, translation_state);
                println!("{:?}", command);

                let mut new_controller = controller;
                let (actions, new_state) = plojo::parse_command(new_state, &dict, command);
                println!("after parsing: {:?}", start.elapsed());
                new_controller.dispatch(actions);

                println!("after dispatching: {:?}", start.elapsed());
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
                vec![plojo::Translation::Text(Text::Lit(
                    translation_str.as_str().unwrap().to_string(),
                ))],
            )
        });

    // NOTE: this fingerspell is just temporary
    fn fingerspell(stroke: &str, letter: &str) -> (plojo::Stroke, Vec<plojo::Translation>) {
        (
            plojo::Stroke::new(stroke),
            vec![
                plojo::Translation::Text(Text::Lit(letter.to_string())),
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
            ],
        )
    }

    // custom strokes added
    let added = vec![
        (
            plojo::Stroke::new("*"),
            vec![plojo::Translation::Command(plojo::Command::Internal(
                plojo::InternalCommand::Undo,
            ))],
        ),
        (
            plojo::Stroke::new("KPA"),
            vec![plojo::Translation::Text(Text::TextAction(vec![
                plojo::TextAction::space(true, true),
                plojo::TextAction::case(true, true),
            ]))],
        ),
        (
            plojo::Stroke::new("KPA*"),
            vec![plojo::Translation::Text(Text::TextAction(vec![
                plojo::TextAction::space(true, false),
                plojo::TextAction::case(true, true),
            ]))],
        ),
        (
            plojo::Stroke::new("-RB"),
            vec![plojo::Translation::Text(Text::TextAction(vec![
                plojo::TextAction::space(true, false),
            ]))],
        ),
        (
            plojo::Stroke::new("S-P"),
            vec![plojo::Translation::Text(Text::TextAction(vec![
                plojo::TextAction::space(true, true),
            ]))],
        ),
        (
            plojo::Stroke::new("TP-PL"),
            vec![
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
                plojo::Translation::Text(Text::Lit(".".to_string())),
                plojo::Translation::Text(Text::TextAction(vec![
                    plojo::TextAction::space(true, true),
                    plojo::TextAction::case(true, true),
                ])),
            ],
        ),
        (
            plojo::Stroke::new("H-F"),
            vec![
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
                plojo::Translation::Text(Text::Lit("?".to_string())),
                plojo::Translation::Text(Text::TextAction(vec![
                    plojo::TextAction::space(true, true),
                    plojo::TextAction::case(true, true),
                ])),
            ],
        ),
        (
            plojo::Stroke::new("SKHRAPL"),
            vec![
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
                plojo::Translation::Text(Text::Lit("!".to_string())),
                plojo::Translation::Text(Text::TextAction(vec![
                    plojo::TextAction::space(true, true),
                    plojo::TextAction::case(true, true),
                ])),
            ],
        ),
        (
            plojo::Stroke::new("KW-BG"),
            vec![
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
                plojo::Translation::Text(Text::Lit(",".to_string())),
            ],
        ),
        (
            plojo::Stroke::new("TK-FPS"),
            vec![plojo::Translation::Text(Text::TextAction(vec![
                plojo::TextAction::space(false, false),
            ]))],
        ),
        (
            plojo::Stroke::new("KW-GS"),
            vec![
                plojo::Translation::Text(Text::Lit("\"".to_string())),
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
            ],
        ),
        (
            plojo::Stroke::new("KR-GS"),
            vec![
                plojo::Translation::Text(Text::TextAction(vec![plojo::TextAction::space(
                    true, false,
                )])),
                plojo::Translation::Text(Text::Lit("\"".to_string())),
            ],
        ),
        fingerspell("A*", "a"),
        fingerspell("PW*", "b"),
        fingerspell("KR*", "c"),
        fingerspell("TK*", "d"),
        fingerspell("*E", "e"),
        fingerspell("TP*", "f"),
        fingerspell("TKPW*", "g"),
        fingerspell("H*", "h"),
        fingerspell("*EU", "i"),
        fingerspell("SKWR*", "j"),
        fingerspell("K*", "k"),
        fingerspell("HR*", "l"),
        fingerspell("PH*", "m"),
        fingerspell("TPH*", "n"),
        fingerspell("O*", "o"),
        fingerspell("P*", "p"),
        fingerspell("KW*", "q"),
        fingerspell("R*", "r"),
        fingerspell("S*", "s"),
        fingerspell("T*", "t"),
        fingerspell("*U", "u"),
        fingerspell("SR*", "v"),
        fingerspell("W*", "w"),
        fingerspell("KP*", "x"),
        fingerspell("KWR*", "y"),
        fingerspell("STKPW*", "z"),
        fingerspell("A*P", "A"),
        fingerspell("PW*P", "B"),
        fingerspell("KR*P", "C"),
        fingerspell("TK*P", "D"),
        fingerspell("*EP", "E"),
        fingerspell("TP*P", "F"),
        fingerspell("TKPW*P", "G"),
        fingerspell("H*P", "H"),
        fingerspell("*EUP", "I"),
        fingerspell("SKWR*P", "J"),
        fingerspell("K*P", "K"),
        fingerspell("HR*P", "L"),
        fingerspell("PH*P", "M"),
        fingerspell("TPH*P", "N"),
        fingerspell("O*P", "O"),
        fingerspell("P*P", "P"),
        fingerspell("KW*P", "Q"),
        fingerspell("R*P", "R"),
        fingerspell("S*P", "S"),
        fingerspell("T*P", "T"),
        fingerspell("*UP", "U"),
        fingerspell("SR*P", "V"),
        fingerspell("W*P", "W"),
        fingerspell("KP*P", "X"),
        fingerspell("KWR*P", "Y"),
        fingerspell("STKPW*P", "Z"),
    ]
    .into_iter();

    plojo::Dictionary::from_iter(parsed.chain(added))
}
