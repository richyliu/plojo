//! Looks up the stroke the dictionary, using a greedy algorithm to convert it into a translation
use super::Dictionary;
use crate::{Text, Translation};
use plojo_core::Stroke;
use std::slice;

// Limit the max number of strokes per translation for performance reasons
// Note: running the following command on the plover dictionary reveals that just 10 translations
// require more than 7 strokes (the max being 10)
// ```
// sed 's/[^\/]//g' plover.json | awk '{ print length }' | sort -nr | head -30
// ```
const MAX_TRANSLATION_STROKE_LEN: usize = 10;

/// Looks up the definition of strokes in the dictionary, converting them into a Translation. Since
/// multiple strokes could map to one dictionary translation, a greedy algorithm is used starting
/// from the oldest strokes. If a stroke is None, it will forcible break up the translation (used
/// for retrospective add space)
pub(super) fn translate_strokes(dict: &Dictionary, strokes: &[Stroke]) -> Vec<Translation> {
    let mut all_translations: Vec<Translation> = vec![];

    let mut start = 0;
    while start < strokes.len() {
        let mut found_translation = false;

        // limit how far to look forward
        let max_end = std::cmp::min(start + MAX_TRANSLATION_STROKE_LEN, strokes.len());

        // look forward up to a certain number of strokes, starting from the most strokes
        for end in (start..max_end).rev() {
            // try suffix folding if it's just the single stroke
            if start == end {
                if let Some(mut translations) = try_suffix_folding(&dict, &strokes[start]) {
                    all_translations.append(&mut translations);
                    start = end + 1;
                    found_translation = true;
                    break;
                }
            }

            // if the strokes give a translation, add it and advance start
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
            all_translations.push(Translation::Text(Text::UnknownStroke(
                strokes[start].clone(),
            )));
            start += 1;
        }
    }

    all_translations
}

// suffixes for suffix folding (currently must all be right hand suffixes)
const SUFFIXES: [&str; 4] = ["-Z", "-D", "-S", "-G"];
// keys used to distinguish right hand keys (for suffix)
const CENTER_KEYS: [char; 6] = ['*', '-', 'A', 'O', 'E', 'U'];

/// Try to extract a suffix from a stroke (handles "suffix folding")
/// It will check if the resulting stroke and suffix have translations and return that
///
/// For example, "KARS" will return the iook up of "KAR" and "-S" in the dictionary
/// "WORLD" will return None because there is no suffix to remove
fn try_suffix_folding(dict: &Dictionary, stroke: &Stroke) -> Option<Vec<Translation>> {
    // if the original stroke has a translation, don't extract suffixes
    if let Some(t) = dict.lookup(slice::from_ref(stroke)) {
        return Some(t);
    }

    let raw_stroke = stroke.clone().to_raw();
    // ignore stroke if it doesn't contains right hand keys (since all suffixes are right hand)
    // this is detected with middle keys, which must be present if there are right hand keys
    if let Some(center_loc) = raw_stroke.find(&CENTER_KEYS[..]) {
        // try each suffix in order
        for s in SUFFIXES.iter() {
            // get the suffix (ignore the leading dash)
            let suffix_char = &s[1..2];
            // check if the suffix exists in the stroke (after the center strokes)
            if raw_stroke[center_loc..].contains(suffix_char) {
                // remove last occurrence of the suffix
                let reversed: String = raw_stroke.chars().rev().collect();
                // remove at most 1 suffix starting from the end
                let removed_suffix = reversed.replacen(suffix_char, "", 1);
                // remove extraneous dash if there is any
                let removed_suffix = if removed_suffix.starts_with('-') {
                    removed_suffix[1..].to_owned()
                } else {
                    removed_suffix
                };
                let removed_suffix: String = removed_suffix.chars().rev().collect();
                if let Some(base) = dict.lookup(&[Stroke::new(&removed_suffix)]) {
                    if let Some(mut suffix_translation) = dict.lookup(&[Stroke::new(s)]) {
                        let mut t = base;
                        t.append(&mut suffix_translation);
                        return Some(t);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StateAction, Text};
    use plojo_core::Command;

    fn testing_dict() -> Dictionary {
        // handy helper function for making dictionary entries
        fn row(stroke: &str, translation: &str) -> (Stroke, Vec<Translation>) {
            (
                Stroke::new(stroke),
                vec![Translation::Text(Text::Lit(translation.to_string()))],
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
            (row("-S", "s")),
            (row("-G", "ing")),
            (row("PH*PB", "mountain")),
            (
                Stroke::new("KPA"),
                vec![Translation::Text(Text::StateAction(
                    StateAction::ForceCapitalize,
                ))],
            ),
            (
                Stroke::new("TKAO*ER"),
                vec![
                    Translation::Text(Text::Lit("deer and printing hello".to_string())),
                    Translation::Command {
                        cmds: vec![Command::PrintHello],
                        text_after: None,
                        suppress_space_before: false,
                    },
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
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_after: None,
                    suppress_space_before: false,
                },
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
                Translation::Text(Text::StateAction(StateAction::ForceCapitalize))
            ]
        );
    }

    #[test]
    fn test_suffix_folding() {
        fn all_text_helper(text: &[&str]) -> Vec<Translation> {
            let mut translations = Vec::with_capacity(text.len());
            for t in text {
                translations.push(Translation::Text(Text::Lit(t.to_string())));
            }
            translations
        }
        let dict = testing_dict();

        assert_eq!(
            try_suffix_folding(&dict, &Stroke::new("H-LS")).unwrap(),
            all_text_helper(&["Hello", "s"])
        );
        assert_eq!(
            try_suffix_folding(&dict, &Stroke::new("TPAOGD")).unwrap(),
            all_text_helper(&["food", "ing"])
        );
        assert_eq!(
            try_suffix_folding(&dict, &Stroke::new("PH*PBS")).unwrap(),
            all_text_helper(&["mountain", "s"])
        );
        assert!(try_suffix_folding(&dict, &Stroke::new("SH-L")).is_none());
        assert!(try_suffix_folding(&dict, &Stroke::new("TPAOGSD")).is_none());
        assert!(try_suffix_folding(&dict, &Stroke::new("H")).is_none());
        assert!(try_suffix_folding(&dict, &Stroke::new("H-LZ")).is_none());
        assert!(try_suffix_folding(&dict, &Stroke::new("STPAODS")).is_none());
    }
}
