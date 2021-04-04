# Plojo

Note: uses [enigo](https://crates.io/crates/enigo) for computer control. Linux
users may need to install libxdo-dev.

## Immediate todos

- check drop-in compatibility (dictionary format changer)
- write documentation
- for corrections (browser URL bar), add space and then delete
- add orthography (`-Z`) for numbers? (to add "00" to a number)
- call function (like for date)
- mouse control

### Plover compatible

#### Text
- add support for plover modes (caps, camel, snake, etc.)?
- add orthography (`-Z`) for numbers? (to add "00" to a number)
- suffix strokes on multistroke words

#### Other
- support for escaping characters (`\\{`, `\\}`)
- for stroke lookup, search also with first letter capitalized/lowercased
- BUG?: "okay, okay" has 2 spaces
- BUG?: capitalize last word for: (`|` indicates cursor)
  - how should these be handled?
  - `said, 'hi`
  - `get frank's`
- BUG: `1BGS` works but `5BGS` doesn't

### Features

- record and replay macros
- a stroke that can add something after the next stroke
- allow comments to be added to the dictionary

- add support for multiple dictionaries that can have their order changed
- something that can suggest briefs based on usage
  - calculate stroke speed and average strokes per word
  - find which strokes happen quickly one after the other (for brief suggestion)
- translation mode for verbatim strokes for brief creation
- all lowercase text filter for texting
- calculate translation frequency
  - also calculate stroke speed and strokes per day
  - calculate strokes per word
  - calculate frequency of certain individual keys and two keys

## Test on Plover
- `SEUFRPL` + `ER/ROR`
- `EPB/TKOUD` be "endowed" or "endo youed"
- `STREUP/-D` be "stripped" or "striped"
- `UPB/TPA*EUGS` be "unification" or "uniification"
- `TPAUR/OUS` be "forous" or "forrous"
- (space after): `H-L/TK-LS/*` be "hello " or ""
- `R-RS`: be "<return>s" or just "s" (suffix folding for commands?)

## Later todos

### Optimization
- I probably shouldn't worry about performance because it is already really fast
- use a bloom filter to prevent need to lookup a long stroke
  - instead of looking up 10, 9, 8, ... 1 strokes joined together
  - 10..n (where n is around 4) could be looked up in a bloom filter
    - could be done in parallel as well
- store prev_strokes in a VecDeque instead of a Vec
  - only diff the last 15 or something strokes instead of all the strokes
- find out what text was deleted to allow for delete by word optimization
- limit number of strokes sent to `translate_strokes`
- possibly optimize hashmap lookup by turning steno keys into a u32
- initialize vecs and hashmaps with capacity to improve performance
- alternate log format that stores time deltas and strokes as ints for low size

### Cleanup
- look for plojo config folder in multiple places (instead of just `~/.plojo`)
- write dictionary parsing as a serde deserializer
- check for stroke validity with a regex and warn if a stoke is invalid
- refactor machine to use more traits
- use macros for raw stokes parsing
- implement feature flag for serde deserializing in plojo_core
- consolidate `Lit` and `Attached` (and maybe even `Glued`)

### Plover compatible
- ignore dictionary unknown special actions
- add orthography rules aliases
- add config to customize undo strokes
- some strokes (like `O-RBGS`) have a dash when it doesn't need a dash
- should be usable as a drop-in replacement for Plover

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
  - for something like `©ab`, the `a` will be capitalized: `©Ab`
- `suppress_space_before` is the same as a `{^}` before command in Plover
- retrospective add space will add a space in the stroke buffer
  - this means that undo will "undo" the space stroke that was added
  - retrospective add space itself cannot be undone either
- retrospective add space stroke is set in the config (not with `{*?}`)
- document the overall architecture of plojo
- prefix + suffix doesn't apply orthography (this may differ from Plover)
  - ex: `TPAUR/OUS` makes `forous`, not `forrous`
- force same case overrides force capitalize (when it applies to the next word)
