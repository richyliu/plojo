/// A stroke can be a single stroke (ex: "H-L") or several strokes (ex:
/// "H-L/WORLD")
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Stroke(String);

impl Stroke {
    pub fn new(stroke: &str) -> Self {
        Self(String::from(stroke))
    }

    pub fn to_raw(self) -> String {
        self.0
    }
}

pub fn is_valid_stroke(stroke: &str) -> bool {
    stroke.len() > 0
}
