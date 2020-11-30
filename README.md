# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Todos

- BUG: I can't use some keyboard shortcuts after pressing arrow
  - command + shortcut sometimes doesn't work either
  - maybe try using autopilot again
  - https://github.com/autopilot-rs/autopilot-rs/blob/master/src/key.rs#L365
  - https://github.com/openstenoproject/plover/blob/master/plover/oslayer/osxkeyboardcontrol.py#L131
  - https://github.com/enigo-rs/enigo/blob/master/src/macos/macos_impl.rs#L221
  - https://developer.apple.com/documentation/coregraphics/cgeventsourcestateid

- run shell commands with stroke
  - https://doc.rust-lang.org/std/process/struct.Command.html#method.spawn
- add option for text actions to be appended after a command
- suffixes folding (-D, -S, -Z, -G) (make sure their order is good)
- undo should remove all the text actions if there are any
  - it should also remove all the commands

- use an english dictionary lookup to fix orthography errors
  - BUG: `SHEUFR/-G` gives "shiverring"; need to use a dictionary for orthography
    - or maybe don't and write a warning for that
    - "summitting" also got the consonant doubled (it should be "summiting")
    - maybe check with the dictionary a "simple" join first
    - also "victorry"
    - https://github.com/openstenoproject/plover/blob/master/plover/orthography.py
    - the issue is primarily with consonant doubling
- add custom stroke/command to also reset `prev_stroke`
- implement `{}` to clear formatting
  - implement it as a suppress space followed by an empty string
- actually implement carrying capitalization
- fix retrospective add/remove space to work on the previous stroke, not word
- upper/lower casing entire words
- clear `prev_stroke` after a command?
- write a script to convert plover shortcut keys to plojo keys

- escape sequences (especially for brackets) in dictionary
- ignore dictionary unknown special actions
- store prev_strokes in a VecDeque instead of a Vec
  - only diff the last 15 or something strokes instead of all the strokes
- find out what text was deleted to allow for delete by word optimization
- add orthography rules aliases
- sort orthography rules in the order of most to least used
- limit number of strokes sent to `translate_strokes`
- check for stroke validity with a regex and warn if a stoke is invalid
- add option for inserting spaces after instead of before
- refactor machine to use more traits
- potential bug: uppercase the next word (without specifying space) and then
  stroking an attached word results in that word *not* being attached (space is
  not suppressed)
  - this is because an attached stroke (by itself) is only attached to the
    previous word if it is the first word
- write somewhere about how commands are dispatched without modifying any text
  - even if a correction is required, it will not press any backspaces
  - command will only be dispatched if it has been newly added
- document the keys available for pressing and how raw key codes are allowed
- consider changing commands format back to one that is plover compatible
- grep for all the NOTEs and document them
- initialize vecs and hashmaps with capacity to improve performance
- note that numbers in a stroke must have a dash where necessary
  - if 0/5 not present and there are digits 6-9
- note that translations with only numbers will be interpreted as glued
