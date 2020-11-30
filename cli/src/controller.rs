use plojo_core::Command;

mod enigo;

pub use self::enigo::EnigoController;

pub trait Controller {
    fn new() -> Self
    where
        Self: Sized;
    fn dispatch(&mut self, command: Command);
}
