# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Todos

These todos are not as important as the ones directly in the code

- change dictionary format
- BUG: word + suffix + punctuation separates the word
- BUG: `UPB/HRES/-T` gives "un lessthe" instead of "unless the"
- BUG: `H-L/TK-LS/*PLT/KW-BG` gives "hello ing," instead of "helloing,"
  - this occurs with **suppressing space before any suffix stroke**
- BUG: `SHEUFR/-G` gives "shiverring"; need to use a dictionary for orthography
  - or maybe don't and write a warning for that
  - "summitting" also got the consonant doubled (it should be "summiting")
  - maybe check with the dictionary a "simple" join first
- BUG: period followed by quote does not join together
  - also: period -> suppress space -> quote -> KPA creates extra space
- BUG: number followed by attached stroke does not work
- BUG: suppress space should lowercase the next word as well
  - maybe make `{^^}` as a join of empty string to clear formatting
  - is `{^}` even supported (or even the same as `{^^}`)?
- BUG: `SEUT/H-PB/TPH/-S` makes "sit-s in"
- BUG: `TP-PL/KR-GS` makes "`. "`" instead of "`."`"
- instead of `{}` to clear formatting, add custom stroke to also reset buffer
- ability to load multiple dictionaries
- external key presses (arrows, control, command, etc)
- glue operator (for fingerspelling, number keys)
- suffixes folding (-D, -S, -Z, -G) (make sure their order is good)
- undo should remove all the text actions if there are any
- stroke that resets `prev_stroke` (similar in function to `{}` in Plover)
- actually implement carrying capitalization
- fix retrospective add/remove space to work on the previous stroke, not word
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
- add option for inserting spaces after instead of before
- refactor machine to use more traits
