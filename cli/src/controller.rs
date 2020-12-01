use plojo_core::Command;

mod applescript;
mod autopilot;
mod enigo;

pub use self::applescript::ApplescriptController;
pub use self::autopilot::AutopilotController;
pub use self::enigo::EnigoController;

pub trait Controller {
    fn new() -> Self
    where
        Self: Sized;
    fn dispatch(&mut self, command: Command);
}
