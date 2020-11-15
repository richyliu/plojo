/// What action should be taken

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Replace(usize, String),
}

impl Command {
    pub fn add_text(output: &str) -> Self {
        Self::replace_text(0, output)
    }
    pub fn replace_text(backspace_num: usize, replace_str: &str) -> Self {
        Self::Replace(backspace_num, replace_str.to_owned())
    }
}
