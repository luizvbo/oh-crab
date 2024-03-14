#!/bin/bash

uninstalled_command() {
  echo "Command \"uninstalled_command\" not found";
}
conda() {
  echo -e "CommandNotFoundError: No command 'conda lst'.\nDid you mean 'conda list'?";
}
ag() {
  echo "...run ag with -Q";
}

