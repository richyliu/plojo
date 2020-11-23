use crate::{Command, Stroke, Translator};
use dictionary::Dictionary;
use diff::translation_diff;
use std::error::Error;

mod dictionary;
mod diff;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct TextAction {
    action_type: TextActionType,
    // associated value for each text action (see TextActionType documentation)
    val: bool,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum TextActionType {
    // true to force a space, false for no space
    SpaceNext,
    SpacePrev,
    // true for uppercase, false for lowercase
    CaseNext,
    CasePrev,
}

impl TextAction {
    fn space(is_next: bool, val: bool) -> Self {
        Self {
            action_type: if is_next {
                TextActionType::SpaceNext
            } else {
                TextActionType::SpacePrev
            },
            val,
        }
    }

    fn case(is_next: bool, val: bool) -> Self {
        Self {
            action_type: if is_next {
                TextActionType::CaseNext
            } else {
                TextActionType::CasePrev
            },
            val,
        }
    }
}

/// A dictionary entry. It could be a command, in which case it is passed directly to the
/// dispatcher. Otherwise it is something that pertains to text, which is parsed here in translator
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
enum Translation {
    Text(Text),
    Command(Command),
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
enum Text {
    // text literal that can be upper/lower cased
    Lit(String),
    // unknown strokes always printed in all caps
    UnknownStroke(Stroke),
    // an attached string that gets orthographic rules applied
    Attached(String),
    // actions like no space, uppercase; apply to adjacent Texts
    TextAction(Vec<TextAction>),
}

impl Translation {
    fn as_text(&self) -> Option<Text> {
        match self {
            Translation::Text(ref text) => Some(text.clone()),
            _ => None,
        }
    }
}

/// The standard translator is very similar in feature to Plover and other CAT software.
///
/// It translates a stroke into a command by looking up the stroke in a dictionary. It maintains a
/// history of pressed strokes and tries to look up the longest stroke in the dictionary.
#[derive(Debug, PartialEq)]
pub struct StandardTranslator {
    prev_strokes: Vec<Stroke>,
    dict: Dictionary,
}

// most number of strokes to stroke in prev_strokes; limits undo to this many strokes
const MAX_STROKE_BUFFER: usize = 100;

/// The configuration for the standard translator
///
/// Creating the translator will take a raw dictionary string (read from a JSON file) and try to
/// parse it. The starting strokes will be added to the stroke list when the translator is created
pub struct Config {
    raw_dict: String,
    starting_strokes: Vec<Stroke>,
}

impl Config {
    /// Creates a config for creating a standard translator.
    pub fn new(raw_dict: String, starting_strokes: Vec<Stroke>) -> Self {
        Self {
            raw_dict,
            starting_strokes,
        }
    }
}

impl Translator for StandardTranslator {
    type T = Config;

    fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let dict = Dictionary::new(&config.raw_dict)?;
        Ok(Self {
            prev_strokes: config.starting_strokes,
            dict,
        })
    }

    fn translate(&mut self, stroke: Stroke) -> Command {
        if self.prev_strokes.len() > MAX_STROKE_BUFFER {
            self.prev_strokes.remove(0);
        }

        let old_translations = self.dict.translate(&self.prev_strokes);
        self.prev_strokes.push(stroke);
        let new_translations = self.dict.translate(&self.prev_strokes);

        translation_diff(&old_translations, &new_translations)
    }

    fn undo(&mut self) -> Command {
        let old_translations = self.dict.translate(&self.prev_strokes);
        self.prev_strokes.pop();
        let new_translations = self.dict.translate(&self.prev_strokes);

        translation_diff(&old_translations, &new_translations)
    }
}
