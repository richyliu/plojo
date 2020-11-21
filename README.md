# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Todos

These todos are not as important as the ones directly in the code

- BUG: consecutive `S-P` produces 2 spaces
  - really need to add integration tests
- ability to load multiple dictionaries
- external key presses (arrows, control, command, etc)
- glue operator (for fingerspelling, number keys)
- suffixes folding (-D, -S, -Z, -G) (make sure their order is good)
- undo should remove all the text actions if there are any
- stroke that resets `prev_stroke` (similar in function to `{}` in Plover)
- actually implement carrying capitalization
- fix retrospective add/remove space to work on the previous stroke, not word
- organize imports better
- escape sequences (especially for brackets) in dictionary
- ignore dictionary unknown special actions
- upper/lower casing entire words
- store prev_strokes in a VecDeque instead of a Vec
- find out what text was deleted to allow for delete by word optimization
- add orthography rules aliases
- for translation, check if 1 old command is replaced by 2 new commands
- sort orthography rules in the order of most to least used
- limit number of strokes sent to `translate_strokes`
- check for stroke validity with a regex
