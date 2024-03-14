#!/bin/bash

# Array of command + expected output pairs. The pair is separated by ":"
tests=(
  "ag test:ag -Q test"
  "uninstalled_command:sudo apt-get install uninstalled_command"
  "cd abcdef:mkdir -p abcdef && cd abcdef"
  "conda lst:conda list"
)

# Iterate over the array
for test in "${tests[@]}"; do
  # Split the test into command and expected_output
  IFS=":" read -r command expected_output <<< "$test"

  # Run the command and pipe the output to a while loop
  cargo run -- -e "source mocked_cli.sh" -- "$command" 2>&1 | while IFS= read -r line
  do
    # Print the line
    # echo "$line"

    # Check if the line starts with "Candidate command(s): ["
    if [[ "$line" =~ "Candidate command(s): ["* ]]; then
      # Extract the candidate command from the line
      candidate_command=$(echo "$line" | awk -F'\\["|\\"]' '{print $2}')
      # Check if the candidate command matches the expected output
      if [[ "$candidate_command" == *"$expected_output"* ]]; then
        echo -e "\033[0;32mTest passed: \"$command\" > \"$expected_output\"\033[0m"
      else
        echo -e "\033[0;31mTest failed: \"$command\" > \"$candidate_command\" != \"$expected_output\"\033[0m"
      fi

      # Kill the cargo run process using killall
      killall ohcrab
      break
    fi
  done
done
