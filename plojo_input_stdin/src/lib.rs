use plojo_core::{Machine, Stroke};
use std::{error::Error, io, io::Write};

pub struct StdinMachine {}

impl StdinMachine {
    pub fn new() -> Self {
        Self {}
    }
}

impl Machine for StdinMachine {
    fn read(&mut self) -> Result<Stroke, Box<dyn Error>> {
        let mut stroke = Stroke::new("");

        // keep prompting the user until the stroke is valid
        while !stroke.is_valid() {
            // prompt the user to provide a stroke
            print!("Stroke> ");
            io::stdout().flush()?;

            let mut input = String::new();
            // blocks until input is read
            io::stdin().read_line(&mut input)?;

            stroke = Stroke::new(&input.trim());
        }

        Ok(stroke)
    }

    fn disable(&self) {
        // no point in disabling stdin machine
    }
}
