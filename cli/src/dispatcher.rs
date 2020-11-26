use translator::{Command, ExternalCommand, InternalCommand};

mod controller;

pub use controller::{Controller, ControllerAction};

const BACKSPACE_DELAY: u32 = 2;
const KEY_DELAY: u32 = 5;

/// Parse the new command into a list of controller actions
pub fn parse_command(command: Command) -> Vec<ControllerAction> {
    let mut actions = vec![];

    match command {
        Command::Internal(internal_command) => {
            let mut new_actions = parse_internal_command(internal_command);
            actions.append(&mut new_actions);
        }
        Command::External(external_command) => {
            let mut new_actions = parse_external_command(external_command);
            actions.append(&mut new_actions);
        }
        Command::NoOp => {}
    }

    actions
}

fn parse_external_command(command: ExternalCommand) -> Vec<ControllerAction> {
    let mut actions = vec![];
    match command {
        ExternalCommand::Replace(num_backspace, add_text) => {
            if num_backspace > 0 {
                actions.push(ControllerAction::BackspaceWithDelay(
                    num_backspace,
                    BACKSPACE_DELAY,
                ));
            }

            if add_text.len() > 0 {
                actions.push(ControllerAction::TypeWithDelay(add_text, KEY_DELAY));
            }
        }
        ExternalCommand::PrintHello => {
            println!("Hello!");
        }
    }

    actions
}

fn parse_internal_command(command: InternalCommand) -> Vec<ControllerAction> {
    // currently no internal commands
    match command {
        _ => vec![],
    }
}
