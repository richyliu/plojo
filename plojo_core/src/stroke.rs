use serde::Deserialize;

/// A steno stroke. Can be a single stroke (ex: "H-L") or several strokes (ex: "H-L/WORLD")
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct Stroke(String);

impl Stroke {
    pub fn new(stroke: &str) -> Self {
        Self(String::from(stroke))
    }

    pub fn to_raw(self) -> String {
        self.0
    }

    pub fn is_undo(&self) -> bool {
        self.0.len() == 1 && self.0.clone() == "*"
    }

    pub fn is_valid(&self) -> bool {
        self.0.len() > 0
    }
}
