#!/bin/bash

# Define an array of command and expected output pairs
declare -A tests=( ["cd abcdef"]="mkdir -p abcdef && cd abcdef" ["cd ghi"]="mkdir -p ghi && cd ghi" )

# Iterate over the array
for command in "${!tests[@]}"; do
  expected_output="${tests[$command]}"

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
        echo "Test passed for command: $command"
      else
        echo "Test failed for command: $command"
        echo "Expected: $expected_output"
        echo "Got: $candidate_command"
      fi

      # Kill the cargo run process using killall
      killall ohcrab
      break
    fi
  done
done
