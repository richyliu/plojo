//! Looks up the stroke the dictionary, using a greedy algorithm to convert it into a translation
use super::Dictionary;
use crate::{Text, Translation};
use plojo_core::Stroke;

// Limit the max number of strokes per translation for performance reasons
// Note: running the following command on the plover dictionary reveals that just 10 translations
// require more than 7 strokes (the max being 10)
// ```
// sed 's/[^\/]//g' plover.json | awk '{ print length }' | sort -nr | head -30
// ```
const MAX_TRANSLATION_STROKE_LEN: usize = 15;

/// Looks up the definition of strokes in the dictionary, converting them into a Translation. Since
/// multiple strokes could map to one dictionary translation, a greedy algorithm is used starting
/// from the oldest strokes
pub(super) fn translate_strokes(dict: &Dictionary, strokes: &Vec<Stroke>) -> Vec<Translation> {
    let mut all_translations: Vec<Translation> = vec![];

    // limit how far to look forward
    let add_max_limit_len = |n: usize| -> usize {
        let with_max = n + MAX_TRANSLATION_STROKE_LEN;
        if with_max > strokes.len() {
            strokes.len()
        } else {
            with_max
        }
    };

    let mut start = 0;
    while start < strokes.len() {
        let mut found_translation = false;
        // look forward up to a certain number of strokes, starting from the most strokes
        for end in (start..add_max_limit_len(start)).rev() {
            // if that gives a translation, add it and advance start
            if let Some(mut translations) = dict.lookup(&strokes[start..=end]) {
                all_translations.append(&mut translations);
                start = end + 1;
                found_translation = true;
                break;
            }
        }

        // if no translation found for any stroke from [start..=start] to [start..=start + max]
        if !found_translation {
            // translation for this stroke
            if let Some(s) = strokes.get(start) {
                all_translations.push(Translation::Text(Text::UnknownStroke(s.clone())));
                start += 1;
            }
        }
    }

    all_translations
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Text, TextAction};
    use plojo_core::Command;

    fn testing_dict() -> Dictionary {
        // handy helper function for making dictionary entries
        fn row(stroke: &str, translation: &str) -> (Stroke, Vec<Translation>) {
            (
                Stroke::new(stroke),
                vec![Translation::Text(Text::Lit(translation.to_string()))],
            )
        }

        fn row_ta(stroke: &str, text_actions: Vec<TextAction>) -> (Stroke, Vec<Translation>) {
            (
                Stroke::new(stroke),
                vec![Translation::Text(Text::TextAction(text_actions))],
            )
        }

        vec![
            (row("H-L", "Hello")),
            (row("WORLD", "World")),
            (row("H-L/A", "He..llo")),
            (row("A", "Wrong thing")),
            (row("TPHO/WUPB", "no one")),
            (row("KW/A/TP", "request an if")),
            (row("H-L/A/WORLD", "hello a world")),
            (row("KW/H-L/WORLD", "request a hello world")),
            (row("PWEUG", "big")),
            (row("PWEUG/PWOEU", "Big Boy")),
            (row("TPAOD", "food")),
            (row_ta(
                "KPA",
                vec![TextAction::space(true, true), TextAction::case(true, true)],
            )),
            (row_ta(
                "KPA*",
                vec![TextAction::space(true, false), TextAction::case(true, true)],
            )),
            (row_ta("-RB", vec![TextAction::space(true, false)])),
            (row_ta("S-P", vec![TextAction::space(true, true)])),
            (
                Stroke::new("H*L"),
                vec![Translation::Command(vec![Command::PrintHello])],
            ),
            (
                Stroke::new("TKAO*ER"),
                vec![
                    Translation::Text(Text::Lit("deer and printing hello".to_string())),
                    Translation::Command(vec![Command::PrintHello]),
                ],
            ),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn test_basic() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("H-L")];
        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![Translation::Text(Text::Lit("Hello".to_string()))]
        );
    }

    #[test]
    fn test_multistroke() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("A"), Stroke::new("H-L")];
        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![
                Translation::Text(Text::Lit("Wrong thing".to_string())),
                Translation::Text(Text::Lit("Hello".to_string()))
            ]
        );
    }

    #[test]
    fn test_correction() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("A")];
        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![Translation::Text(Text::Lit("He..llo".to_string()))]
        );
    }

    #[test]
    fn test_correction_with_history() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("WORLD"), Stroke::new("H-L"), Stroke::new("A")];
        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![
                Translation::Text(Text::Lit("World".to_string())),
                Translation::Text(Text::Lit("He..llo".to_string()))
            ]
        );
    }

    #[test]
    fn test_unknown_stroke() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("SKWR")];
        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![Translation::Text(Text::UnknownStroke(Stroke::new("SKWR")))]
        );
    }

    #[test]
    fn test_all_unknown_stroke() {
        let dict = testing_dict();
        let strokes = vec![
            Stroke::new("TPHO"),
            Stroke::new("TPHOU"),
            Stroke::new("TPHOUT"),
        ];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![
                Translation::Text(Text::UnknownStroke(Stroke::new("TPHO"))),
                Translation::Text(Text::UnknownStroke(Stroke::new("TPHOU"))),
                Translation::Text(Text::UnknownStroke(Stroke::new("TPHOUT")))
            ]
        );
    }

    #[test]
    fn test_multi_unknown_stroke() {
        let dict = testing_dict();
        let strokes = vec![
            Stroke::new("TPHO"),
            Stroke::new("TPHOU"),
            Stroke::new("TPHO"),
            Stroke::new("WUPB"),
        ];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![
                Translation::Text(Text::UnknownStroke(Stroke::new("TPHO"))),
                Translation::Text(Text::UnknownStroke(Stroke::new("TPHOU"))),
                Translation::Text(Text::Lit("no one".to_string()))
            ]
        );
    }

    #[test]
    fn test_middle_unknown() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("A"), Stroke::new("WORLD")];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![Translation::Text(Text::Lit("hello a world".to_string()))]
        );
    }

    #[test]
    fn test_around_unknown() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("KW"), Stroke::new("A"), Stroke::new("TP")];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![Translation::Text(Text::Lit("request an if".to_string()))]
        );
    }

    #[test]
    fn test_beginning_unknown() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("KW"), Stroke::new("H-L"), Stroke::new("WORLD")];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![Translation::Text(Text::Lit(
                "request a hello world".to_string()
            ))]
        );
    }

    #[test]
    fn test_multiple_translations() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("TKAO*ER")];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("deer and printing hello".to_string())),
                Translation::Command(vec![Command::PrintHello]),
            ]
        );
    }

    #[test]
    fn test_text_actions() {
        let dict = testing_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("KPA")];

        let translations = translate_strokes(&dict, &strokes);

        assert_eq!(
            translations,
            vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::TextAction(vec![
                    TextAction::space(true, true),
                    TextAction::case(true, true)
                ])),
            ]
        );
    }
}
