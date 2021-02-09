#!/usr/bin/env sh

# Run this before committing
# A non zero exit code means that the code should *not* be committed

# Runs the unit tests
cargo test --workspace -q &&
# Checks for dbg! macros still left in the code
! ag --ignore-dir cli/runtime_files --ignore check.sh dbg\!
# Checks for TODO's still left in the code
! ag --ignore-dir cli/runtime_files --ignore check.sh 'TODO:'
