#!/bin/bash

# This is usefull to test args scrolling

# Check for a specific argument to know if we are in the "sleeper" stage.
# This prevents an infinite loop of re-execution.
if [ "$1" = "--sleeping" ]; then
  # We are the long-argument process. Just sleep.
  echo "Process $$ is now sleeping with many arguments. Check 'ps' in another terminal."
  sleep 60
  exit
fi

# --- Main execution starts here ---
echo "Preparing to launch process with long argument list..."

# Generate a long list of arguments.
# You can increase the number in {1..200} for an even longer list.
ARGS=()
for i in {1..200}; do
  ARGS+=("argument-number-$i")
done

# Use 'exec' to replace the current script's process image with a new one,
# passing our generated arguments.
# "$0" refers to the current script's path.
exec "$0" --sleeping "${ARGS[@]}"
