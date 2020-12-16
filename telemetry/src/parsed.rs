use serde::{Deserialize, Serialize};

pub type Stroke = String;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: i64,
    pub stroke: Stroke,
    pub content: Content,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Content {
    Replace { backspace_num: u32, text: String },
    Command,
    NoOp,
}
