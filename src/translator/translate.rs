//! Looks up the stroke the dictionary, using a greedy algorithm to convert it into a translation
use crate::stroke::Stroke;
use crate::translator::Dictionary;
use crate::translator::Translation;

// Limit the max number of strokes per translation for performance reasons
const MAX_TRANSLATION_STROKE_LEN: usize = 15;

/// Looks up the definition of strokes in the dictionary, converting them into a Translation. Since
/// multiple strokes could map to one dictionary translation, a greedy algorithm is used starting
/// from the oldest strokes
pub fn translate_strokes(strokes: &Vec<Stroke>, dict: &Dictionary) -> Vec<Translation> {
    let mut translations: Vec<Translation> = vec![];

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
            if let Some(translation) = dict.lookup(&strokes[start..=end]) {
                translations.push(translation);
                start = end + 1;
                found_translation = true;
                break;
            }
        }

        // if no translation found for any stroke from [start..=start] to [start..=start + max]
        if !found_translation {
            // translation for this stroke
            if let Some(s) = strokes.get(start) {
                translations.push(Translation::UnknownStroke(s.clone()));
                start += 1;
            }
        }
    }

    translations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_dict() -> Dictionary {
        Dictionary::new(vec![
            (Stroke::new("H-L"), Translation::text("Hello")),
            (Stroke::new("WORLD"), Translation::text("World")),
            (Stroke::new("H-L/A"), Translation::text("He..llo")),
            (Stroke::new("A"), Translation::text("Wrong thing")),
            (Stroke::new("TPHO/WUPB"), Translation::text("no one")),
            (Stroke::new("KW/A/TP"), Translation::text("request an if")),
            (
                Stroke::new("H-L/A/WORLD"),
                Translation::text("hello a world"),
            ),
            (
                Stroke::new("KW/H-L/WORLD"),
                Translation::text("request a hello world"),
            ),
        ])
    }

    #[test]
    fn test_translator_basic() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("H-L")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(translations, vec![Translation::text("Hello")]);
    }

    #[test]
    fn test_translator_multistroke() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("A"), Stroke::new("H-L")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::text("Wrong thing"), Translation::text("Hello")]
        );
    }

    #[test]
    fn test_translator_correction() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("A")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(translations, vec![Translation::text("He..llo")]);
    }

    #[test]
    fn test_translator_correction_with_history() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("WORLD"), Stroke::new("H-L"), Stroke::new("A")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::text("World"), Translation::text("He..llo")]
        );
    }

    #[test]
    fn test_translator_unknown_stroke() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("SKWR")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::UnknownStroke(Stroke::new("SKWR"))]
        );
    }

    #[test]
    fn test_translator_all_unknown_stroke() {
        let dict = setup_dict();
        let strokes = vec![
            Stroke::new("TPHO"),
            Stroke::new("TPHOU"),
            Stroke::new("TPHOUT"),
        ];

        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![
                Translation::UnknownStroke(Stroke::new("TPHO")),
                Translation::UnknownStroke(Stroke::new("TPHOU")),
                Translation::UnknownStroke(Stroke::new("TPHOUT"))
            ]
        );
    }

    #[test]
    fn test_translator_multi_unknown_stroke() {
        let dict = setup_dict();
        let strokes = vec![
            Stroke::new("TPHO"),
            Stroke::new("TPHOU"),
            Stroke::new("TPHO"),
            Stroke::new("WUPB"),
        ];

        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![
                Translation::UnknownStroke(Stroke::new("TPHO")),
                Translation::UnknownStroke(Stroke::new("TPHOU")),
                Translation::Text("no one".to_string())
            ]
        );
    }

    #[test]
    fn test_translator_middle_unknown() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("A"), Stroke::new("WORLD")];

        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::Text("hello a world".to_string())]
        );
    }

    #[test]
    fn test_translator_around_unknown() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("KW"), Stroke::new("A"), Stroke::new("TP")];

        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::Text("request an if".to_string())]
        );
    }

    #[test]
    fn test_translator_beginning_unknown() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("KW"), Stroke::new("H-L"), Stroke::new("WORLD")];

        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::Text("request a hello world".to_string())]
        );
    }
}