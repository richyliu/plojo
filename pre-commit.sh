#!/usr/bin/env sh

# Run tests and exit if tests failed
./run_tests.sh
RESULT=$?
[ $RESULT -ne 0 ] && exit 1

# Checks for dbg! macros still left in the code
ag --ignore-dir cli/runtime_files --ignore pre-commit.sh "dbg!" &&
  echo 'COMMIT REJECTED Found TODOs. Please remove them before committing.' &&
  exit 1
# Checks for TODOs still left in the code
ag --ignore-dir cli/runtime_files --ignore pre-commit.sh "TODO:" &&
  echo 'COMMIT REJECTED Found TODOs. Please remove them before committing.' &&
  exit 1

exit 0
