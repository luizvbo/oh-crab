#!/bin/bash

# Define the command and expected output
command="cd abcdef"
expected_output="mkdir -p abcdef && cd abcdef"

# Run the command and pipe the output to a while loop
cargo run -- "$command" 2>&1 | while IFS= read -r line
do
  # Print the line
  # echo "$line"

  # Check if the line starts with "Candidate command(s): ["
  if [[ "$line" == Candidate\ command\(s\):\ [* ]]; then
    # Extract the candidate command from the line
    candidate_command=$(echo "$line" | awk -F'\\["|\\"]' '{print $2}')

    # Check if the candidate command matches the expected output
    if [[ "$candidate_command" == "$expected_output" ]]; then
      echo "Test passed"
    else
      echo "Test failed"
      echo "Expected: $expected_output"
      echo "Got: $candidate_command"
    fi

    # Kill the cargo run process
    # kill $!
    killall ohcrab
    break
  fi
done
