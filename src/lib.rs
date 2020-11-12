pub mod translator;

use translator::Dictionary;
use translator::State;
use translator::Stroke;
use translator::Output;

pub fn run_translation(input: Vec<String>) {
    let mut state = State::initial();
    let dict = Dictionary::new(vec![
        (Stroke::new("H-L"), Output::text("Hello")),
        (Stroke::new("WORLD"), Output::text("World")),
    ]);

    for stroke in input.iter() {
        let (output, new_state) = translator::translate(Stroke::new(stroke), &dict, state);
        state = new_state;
        println!("Output: {:?}", output);
    }
}
