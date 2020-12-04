#[macro_use]
extern crate lazy_static;

use dictionary::Dictionary;
use diff::translation_diff;
use plojo_core::{Command, Stroke, Translator};
use serde::Deserialize;
use std::error::Error;

mod dictionary;
mod diff;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
struct TextAction {
    action_type: TextActionType,
    // associated value for each text action (see TextActionType documentation)
    val: bool,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
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
    Command {
        cmds: Vec<Command>,
        text_actions: Option<Vec<TextAction>>,
    },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
enum Text {
    // text literal that can be upper/lower cased
    Lit(String),
    // unknown strokes always printed in all caps
    UnknownStroke(Stroke),
    // an attached string that gets orthographic rules applied
    Attached(String),
    // glued strokes only attach to other glued strokes
    Glued(String),
    // actions like no space, uppercase; apply to adjacent Texts
    TextAction(Vec<TextAction>),
}

impl Translation {
    /// Convert translation into text, ignoring commands
    fn as_text(&self) -> Option<Text> {
        match self {
            Translation::Text(ref text) => Some(text.clone()),
            Translation::Command {
                cmds: _,
                text_actions,
            } => text_actions.clone().map(Text::TextAction),
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
const MAX_STROKE_BUFFER: usize = 50;
// only pass a certain number of strokes to be translated
const MAX_TRANSLATION_STROKE_LEN: usize = 10;

/// The configuration for the standard translator
///
/// Creating the translator will take the raw dictionary string from one or more dictionaries. The
/// dictionaries further down in the list can override the earlier dictionaries.
///
/// The starting strokes will be added to the stroke list when the translator is created.
#[derive(Default)]
pub struct Config {
    raw_dicts: Vec<String>,
    starting_strokes: Vec<Stroke>,
}

impl Config {
    /// Creates a config for creating a standard translator.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_raw_dicts(self, raw_dicts: Vec<String>) -> Self {
        Self { raw_dicts, ..self }
    }

    pub fn with_starting_strokes(self, starting_strokes: Vec<Stroke>) -> Self {
        Self {
            starting_strokes,
            ..self
        }
    }
}

impl StandardTranslator {
    /// Remove all strokes that cannot be undone (currently Commands and text actions)
    fn remove_non_undoable_strokes(&mut self) {
        while let Some(stroke) = self.prev_strokes.pop() {
            let translated = self.dict.translate(&vec![stroke]);
            for t in translated {
                // if any stroke contains text (length > 0), stop removing
                // otherwise keep on removing
                match t {
                    Translation::Command {
                        cmds: _,
                        text_actions: _,
                    } => {
                        // keep on removing
                    }
                    Translation::Text(text) => {
                        match text {
                            Text::TextAction(_) => {
                                // keep on removing
                            }
                            Text::Attached(t) => {
                                if t.len() > 0 {
                                    return;
                                }
                            }
                            Text::Glued(t) => {
                                if t.len() > 0 {
                                    return;
                                }
                            }
                            Text::Lit(t) => {
                                if t.len() > 0 {
                                    return;
                                }
                            }
                            Text::UnknownStroke(_) => {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Translator for StandardTranslator {
    type C = Config;

    fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let dict = Dictionary::new(config.raw_dicts)?;
        Ok(Self {
            prev_strokes: config.starting_strokes,
            dict,
        })
    }

    fn translate(&mut self, stroke: Stroke) -> Vec<Command> {
        if self.prev_strokes.len() > MAX_STROKE_BUFFER {
            self.prev_strokes.remove(0);
        }

        // translate only latest strokes
        let start = if self.prev_strokes.len() > MAX_TRANSLATION_STROKE_LEN {
            self.prev_strokes.len() - MAX_TRANSLATION_STROKE_LEN
        } else {
            0
        };

        let old_translations = self.dict.translate(&self.prev_strokes[start..]);
        self.prev_strokes.push(stroke);
        let new_translations = self.dict.translate(&self.prev_strokes[start..]);

        translation_diff(&old_translations, &new_translations)
    }

    fn undo(&mut self) -> Vec<Command> {
        let old_translations = self.dict.translate(&self.prev_strokes);
        self.remove_non_undoable_strokes();
        let new_translations = self.dict.translate(&self.prev_strokes);

        translation_diff(&old_translations, &new_translations)
    }
}
