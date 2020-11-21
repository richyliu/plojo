# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Todos

- glue operator: only joins to other glued strokes (for fingerspelling and numbers)
- make undo more "special" and handle it at a higher level
- undo should remove all strokes until a non-command stroke
- suffixes that can be added to the same stroke (-D, -S, -Z, -G) (make sure their order is good)
- escape sequences (especially for brackets) in dictionary
- ignore dictionary unknown special actions
- upper/lower casing entire words
- store prev_strokes in a VecDeque instead of a Vec
- find out what text was deleted to allow for delete by word optimization
- add orthography rules aliases
- carrying capitalization
- for translation, check if 1 old command is replaced by 2 new commands
- sort orthography rules in the order of most to least used
- limit number of strokes sent to `translate_strokes`
- check for stroke validity with a regex
- organize imports better
- add integration tests
