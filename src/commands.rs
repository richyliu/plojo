/// What action should be taken

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Command {
    Internal(InternalCommand),
    External(ExternalCommand),
    NoOp,
}

/// Internal commands affect the translation state
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum InternalCommand {}

/// External commands create some output to the computer (keyboard press, mouse move, etc.)
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ExternalCommand {
    Replace(usize, String),
}

impl Command {
    pub fn add_text(output: &str) -> Self {
        Self::replace_text(0, output)
    }
    pub fn replace_text(backspace_num: usize, replace_str: &str) -> Self {
        Self::External(ExternalCommand::Replace(
            backspace_num,
            replace_str.to_owned(),
        ))
    }
}
