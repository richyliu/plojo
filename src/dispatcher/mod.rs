use crate::commands::{Command, ExternalCommand, InternalCommand};
use crate::dispatcher::controller::ControllerAction;
use crate::translator::{undo, Dictionary, State};

pub mod controller;

const BACKSPACE_DELAY: u32 = 10;
const KEY_DELAY: u32 = 20;

/// Given a translation state and a dictionary, parse the new command into a list of controller actions and new state
pub fn parse_command(
    state: State,
    dict: &Dictionary,
    command: Command,
) -> (Vec<ControllerAction>, State) {
    let mut new_state = state;
    let mut actions = vec![];

    match command {
        Command::Internal(internal_command) => {
            let (mut new_actions, temp_state) =
                parse_internal_command(new_state, &dict, internal_command);
            new_state = temp_state;
            actions.append(&mut new_actions);
        }
        Command::External(external_command) => {
            let mut new_actions = parse_external_command(external_command);
            actions.append(&mut new_actions);
        }
    }

    (actions, new_state)
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
            println!("\n====================== hello! ======================\n");
        }
    }

    actions
}

fn parse_internal_command(
    state: State,
    dict: &Dictionary,
    command: InternalCommand,
) -> (Vec<ControllerAction>, State) {
    match command {
        InternalCommand::Undo => {
            let (command, new_state) = undo(dict, state);
            return parse_command(new_state, dict, command);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stroke::Stroke;
    use crate::testing_dict;

    #[test]
    fn test_undo_command() {
        // state includes the undo stroke because that was the newest translation which turned into the undo command
        let state = State::with_strokes(vec![Stroke::new("H-L"), Stroke::new("*")]);
        let dict = testing_dict();
        let action = Command::Internal(InternalCommand::Undo);

        let (actions, _new_state) = parse_command(state, &dict, action);
        assert_eq!(
            actions,
            vec![ControllerAction::BackspaceWithDelay(6, BACKSPACE_DELAY)]
        );
    }
}
