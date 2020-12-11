#[macro_use]
extern crate lazy_static;

use dictionary::Dictionary;
use diff::translation_diff;
use plojo_core::{Command, Stroke, Translator};
use serde::Deserialize;
use std::{error::Error, hash::Hash};

mod dictionary;
mod diff;

/// A dictionary entry. It could be a command, in which case it is passed directly to the
/// dispatcher. Otherwise it is something that pertains to text, which is parsed here in translator
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
enum Translation {
    Text(Text),
    Command {
        cmds: Vec<Command>,
        text_after: Option<Vec<Text>>,
    },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, Deserialize)]
enum Text {
    // text literal that can be upper/lower cased
    Lit(String),
    // unknown strokes always printed in all caps
    UnknownStroke(Stroke),
    // a string that can be attached to the previous and/or next word
    Attached {
        // the text itself
        text: String,
        // if it should be attached to the next word
        joined_next: bool,
        /// Whether or not to apply orthography rules and whether to attach to the next word
        /// Some(true) => apply orthography rules and attach
        /// Some(false) => attach only
        /// None => do not attach to the previous word
        do_orthography: Option<bool>,
    },
    // glued strokes only attach to other glued strokes
    Glued(String),
    // changes the state for suppressing space, capitalizing, etc. the next word
    StateAction(StateAction),
    // text actions can only affect the text before it
    // TODO: does this really need to be a vec?
    TextAction(TextAction),
}

impl Translation {
    /// Convert translation into text, ignoring commands
    fn as_text(&self) -> Option<Vec<Text>> {
        match self {
            Translation::Text(ref text) => Some(vec![text.clone()]),
            Translation::Command {
                cmds: _,
                text_after,
            } => text_after.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, Deserialize)]
pub enum StateAction {
    SuppressSpace,
    // TODO: this should also undo suppress space?
    ForceCapitalize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub enum TextAction {
    CapitalizePrev,
    SuppressSpacePrev,
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

impl StandardTranslator {
    /// Remove all strokes that cannot be undone (currently Commands and text actions)
    fn remove_non_undoable_strokes(&mut self) {
        while let Some(stroke) = self.prev_strokes.pop() {
            let translated = self.dict.translate(&vec![stroke]);
            for t in translated {
                // if any stroke contains text (length > 0), stop removing
                // otherwise keep on removing
                match t {
                    Translation::Command { .. } => {
                        // keep on removing
                    }
                    Translation::Text(text) => {
                        match text {
                            Text::TextAction(_) | Text::StateAction(_) => {
                                // keep on removing
                            }
                            Text::Attached { text, .. } => {
                                if text.len() > 0 {
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

    /// Creats a translator that takes the raw dictionary string from one or more dictionaries. The
    /// dictionaries further down in the list can override the earlier dictionaries.
    ///
    /// The starting strokes will be added to the stroke list when the translator is created.
    pub fn new(
        raw_dicts: Vec<String>,
        starting_strokes: Vec<Stroke>,
    ) -> Result<Self, Box<dyn Error>> {
        let dict = Dictionary::new(raw_dicts)?;
        Ok(Self {
            prev_strokes: starting_strokes,
            dict,
        })
    }
}

impl Translator for StandardTranslator {
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
