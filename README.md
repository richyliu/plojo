# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Todos

- issue where I can't use some keyboard shortcuts after pressing arrow
- rename crates to have plojo prefix to prevent name conflicts
  - rename `translator` to `core` (or `plojo_core`)
- glue operator (for fingerspelling, number keys)
- add option for text actions to be appended after a command
- suffixes folding (-D, -S, -Z, -G) (make sure their order is good)
- implement `{}` to clear formatting
- use an english dictionary lookup to fix orthography errors
  - BUG: `SHEUFR/-G` gives "shiverring"; need to use a dictionary for orthography
    - or maybe don't and write a warning for that
    - "summitting" also got the consonant doubled (it should be "summiting")
    - maybe check with the dictionary a "simple" join first
    - also "victorry"
    - https://github.com/openstenoproject/plover/blob/master/plover/orthography.py
    - the issue is primarily with consonant doubling
- add custom stroke/command to also reset `prev_stroke`

- undo should remove all the text actions if there are any
  - it should also skip all the commands
- actually implement carrying capitalization
- fix retrospective add/remove space to work on the previous stroke, not word
- upper/lower casing entire words
- clear `prev_stroke` after a command?
- run shell commands with stroke

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
  - **command will only be dispatched if it has been newly added**
  - write a script to convert plover shortcut keys to plojo keys
- document the keys available for pressing and how raw key codes are allowed
- consider changing commands format back to one that is plover compatible
- grep for all the NOTEs and document them
- initialize vecs and hashmaps with capacity to improve performance
