use crate::commands::Command;

mod controller;

pub use controller::Controller;

pub fn new_controller() -> Controller {
    Controller::new()
}

const BACKSPACE_DELAY: u32 = 10;
const KEY_DELAY: u32 = 50;
pub fn dispatch(controller: &mut Controller, action: Command) {
    match action {
        Command::Replace(num_backspace, add_text) => {
            if num_backspace > 0 {
                controller.backspace(num_backspace, BACKSPACE_DELAY);
            }

            controller.type_with_delay(&add_text, KEY_DELAY);
        }
    }
}
