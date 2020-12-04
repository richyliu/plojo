#!/usr/bin/env sh

# Run this before committing

# Runs the unit tests
cargo test &&
# Checks for dbg! macros still left in the code
! ag --ignore-dir cli/runtime_files --ignore check.sh dbg\!

