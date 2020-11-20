use crate::{Command, Dictionary, InternalCommand, Stroke, Text, TextAction, Translation};
use serde_json;
use std::fs;
use std::iter::FromIterator;

// TODO: check dictionary when loading and give helpful errors
pub fn load(filename: &str) -> Option<Dictionary> {
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
                Stroke::new(stroke_str),
                vec![Translation::Text(Text::Lit(
                    translation_str.as_str().unwrap().to_string(),
                ))],
            )
        });

    // NOTE: this fingerspell is just temporary
    fn fingerspell(stroke: &str, letter: &str) -> (Stroke, Vec<Translation>) {
        (
            Stroke::new(stroke),
            vec![
                Translation::Text(Text::Lit(letter.to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ],
        )
    }

    // custom strokes added
    let added = vec![
        (
            Stroke::new("*"),
            vec![Translation::Command(Command::Internal(
                InternalCommand::Undo,
            ))],
        ),
        (
            Stroke::new("KPA"),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, true),
            ]))],
        ),
        (
            Stroke::new("KPA*"),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, true),
            ]))],
        ),
        (
            Stroke::new("-RB"),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(true, false),
            ]))],
        ),
        (
            Stroke::new("S-P"),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(true, true),
            ]))],
        ),
        (
            Stroke::new("TP-PL"),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit(".".to_string())),
                Translation::Text(Text::TextAction(vec![
                    TextAction::space(true, true),
                    TextAction::case(true, true),
                ])),
            ],
        ),
        (
            Stroke::new("H-F"),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit("?".to_string())),
                Translation::Text(Text::TextAction(vec![
                    TextAction::space(true, true),
                    TextAction::case(true, true),
                ])),
            ],
        ),
        (
            Stroke::new("SKHRAPL"),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit("!".to_string())),
                Translation::Text(Text::TextAction(vec![
                    TextAction::space(true, true),
                    TextAction::case(true, true),
                ])),
            ],
        ),
        (
            Stroke::new("KW-BG"),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit(",".to_string())),
            ],
        ),
        (
            Stroke::new("TK-FPS"),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(false, false),
            ]))],
        ),
        (
            Stroke::new("KW-GS"),
            vec![
                Translation::Text(Text::Lit("\"".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ],
        ),
        (
            Stroke::new("KR-GS"),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit("\"".to_string())),
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
        (
            Stroke::new("-S"),
            vec![Translation::Text(Text::Attached("s".to_string()))],
        ),
    ]
    .into_iter();

    Some(Dictionary::from_iter(parsed.chain(added)))
}

// TODO: add dictionary tests
