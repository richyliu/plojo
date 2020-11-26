# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Bugs

- BUG: `SHEUFR/-G` gives "shiverring"; need to use a dictionary for orthography
  - or maybe don't and write a warning for that
  - "summitting" also got the consonant doubled (it should be "summiting")
  - maybe check with the dictionary a "simple" join first
  - also "victorry"
  - https://github.com/openstenoproject/plover/blob/master/plover/orthography.py
  - the issue is primarily with consonant doubling

## Todos

- external key presses (arrows, control, command, etc)
  - change dictionary format
  - currently loading quite slowly
- for translation, check if 1 old command is replaced by 2 new commands
- instead of `{}` to clear formatting, add custom stroke to also reset buffer
  - stroke that resets `prev_stroke` (similar in function to `{}` in Plover)
- glue operator (for fingerspelling, number keys)
- suffixes folding (-D, -S, -Z, -G) (make sure their order is good)
- undo should remove all the text actions if there are any
- actually implement carrying capitalization
- fix retrospective add/remove space to work on the previous stroke, not word
- escape sequences (especially for brackets) in dictionary
- ignore dictionary unknown special actions
- upper/lower casing entire words
- store prev_strokes in a VecDeque instead of a Vec
  - only diff the last 15 or something strokes instead of all the strokes
- find out what text was deleted to allow for delete by word optimization
- add orthography rules aliases
- sort orthography rules in the order of most to least used
- limit number of strokes sent to `translate_strokes`
- check for stroke validity with a regex
- add option for inserting spaces after instead of before
- refactor machine to use more traits
- potential bug: uppercase the next word (without specifying space) and then
  stroking an attached word results in that word *not* being attached (space is
  not suppressed)
  - this is because an attached stroke (by itself) is only attached to the
    previous word if it is the first word
