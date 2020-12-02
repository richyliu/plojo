use crate::Command;

pub trait Controller {
    fn new() -> Self
    where
        Self: Sized;
    fn dispatch(&mut self, command: Command);
}
