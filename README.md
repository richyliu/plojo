# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Todos

- **start in diff/parser.rs (implement parse_translation)**
- BUG: prefix + suffixes, followed by remove last space, does not work
  - criteria:
    - may need to rethink the system for space next/previous
    - delete last space needs to work even after a prefix
    - remove space before a fingerspelled word
    - add last space needs to add immediately between the last 2 strokes
    - multiple add/suppress space need to work together
    - carrying capitalization
    - suppressing space
  - brainstorming:
    - system for next word state
      - uppercase, lowercase, ALL CAPS/lower
      - strokes like KPA* and KPA change that state
      - the next word reads the state to do the capitalize/lowercase
    - system for prefix/suffix
      - suppresses space where appropriate
      - suffix needs to be special to apply orthography rules
      - enum "Attached" variant
        - can specify bool attacted to prev and next
        - can specify whether or not to use orthography rules
    - retroactive actions
      - after translating to string, go through a second time to apply actions
      - actions can only affect what comes before it
    - retroactive add space
      - needs to be special, because it add a space (and changes a translation)
    - when deciding next word space/uppercase:
      - check if previous said to suppress space
      - check if current (next) said to suppress space
      - check current space state
      - check if current is carrying capitalization
        - then set state to default and change the state for next word
    - steps
      - translate using dictionary
      - join words, use space/case state, change state where necessary
        - keep text (retroactive) actions the same
      - loop again and apply retroactive actions from back to front
- suffixes folding (-D, -S, -Z, -G) (make sure their order is good)
- use an english dictionary lookup to fix orthography errors
  - BUG: `SHEUFR/-G` gives "shiverring"; need to use a dictionary for orthography
    - the issue is primarily with consonant doubling
    - check with a dictionary for a "simple" join first
    - https://github.com/openstenoproject/plover/blob/master/plover/orthography.py

- actually implement carrying capitalization
- add custom stroke/command to also reset `prev_stroke`
- implement `{}` to clear formatting
  - implement it as a suppress space followed by an empty string
- fix retrospective add/remove space to work on the previous stroke, not word
- upper/lower casing entire words
- clear `prev_stroke` after a command?
- add support for multiple dictionaries that can have their order changed
- allow comments to be added to the dictionary

### Optimization
- use a bloom filter to prevent need to lookup a long stroke
  - instead of looking up 10, 9, 8, ... 1 strokes joined together
  - 10..n (where n is around 4) could be looked up in a bloom filter
    - could be done in parallel as well
- store prev_strokes in a VecDeque instead of a Vec
  - only diff the last 15 or something strokes instead of all the strokes
- find out what text was deleted to allow for delete by word optimization
- sort orthography rules in the order of most to least used
- limit number of strokes sent to `translate_strokes`
- possibly optimize hashmap lookup by turning steno keys into a u32
- initialize vecs and hashmaps with capacity to improve performance

### Cleanup
- write dictionary parsing as a serde deserializer
- check for stroke validity with a regex and warn if a stoke is invalid
- refactor machine to use more traits
- use macros for raw stokes parsing
- implement feature flag for serde deserializing in plojo_core

### Plover compatible
- write a script to convert plover shortcut keys to plojo keys
- ignore dictionary unknown special actions
- escape sequences (especially for brackets) in dictionary
- add orthography rules aliases
- potential bug: uppercase the next word (without specifying space) and then
- add option for inserting spaces after instead of before
  stroking an attached word results in that word *not* being attached (space is
  not suppressed)
  - this is because an attached stroke (by itself) is only attached to the
    previous word if it is the first word
- consider changing commands format back to one that is plover compatible

### Documentation
- write somewhere about how commands are dispatched without modifying any text
  - even if a correction is required, it will not press any backspaces
  - command will only be dispatched if it has been newly added
- document the keys available for pressing and how raw key codes are allowed
- grep for all the NOTEs and document them
- note that numbers in a stroke must have a dash where necessary
  - if 0/5 not present and there are digits 6-9
- note that translations with only numbers will be interpreted as glued
- document how undo removes all strokes that only have text actions and commands
  - also removes text (attached, glued) that is empty
- keyboard shortcuts must use the "raw" version (eg: `[`/`]` instead of `{`/`}`)
- capitalize prev will capitalize the previous word that appears on screen
  - for translations with multiple words, the last word will be capitalized
  - if space prev is suppressed, the whole thing will be capitalized
