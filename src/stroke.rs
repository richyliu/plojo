/// A stroke can be a single stroke (ex: "H-L") or several strokes (ex:
/// "H-L/WORLD")
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Stroke(String);

impl Stroke {
    pub fn new(stroke: &str) -> Self {
        Self(String::from(stroke))
    }

    pub fn empty_stroke() -> Self {
        Self::new("")
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn to_raw(self) -> String {
        self.0
    }

    /// Join a copy of two strokes together with a `/` in the middle. If either stroke is empty,
    /// return the other stroke
    pub fn join(&self, other: &Self) -> Self {
        if other.0.len() == 0 {
            (*self).clone()
        } else if self.0.len() == 0 {
            (*other).clone()
        } else {
            Self::new(&format!("{}/{}", self.0, other.0))
        }
    }
}
