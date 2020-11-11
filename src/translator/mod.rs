use std::collections::HashMap;

#[rustfmt::skip]
enum Key {
    NUMBER,
    SL, TL, PL, HL, STAR, FR, PR, LR, TR, DR,
        KL, WL, RL,       RR, BR, GR, SR, ZR,
             A,  O,       E,  U,
}

struct Stroke {
    keys: Vec<Key>,
}

impl Stroke {
    fn new() -> Stroke {

    }
}

struct Output {
    content: String,
}

struct Dictionary {
    strokes: HashMap<Stroke, Output>,
}

/// Translate needs: stroke, temporary state (previous strokes, next word
/// uppercase, etc), persistent state (dictionary(ies))
pub fn translate() {
    println!("Hello");
}
