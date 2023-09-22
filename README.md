# OhCrab!

`ohcrab` is a port of the well known CLI tool
[`thefuck`](https://github.com/nvbn/thefuck) to our beloved Rust language.

## Table of contents

1. [Installation](#installation)
1. [Usage](#usage)

## Installation

### Installing the package

You can install `ohcrab` using [`cargo`](https://crates.io/):

```bash
cargo install ohcrab
```

### Exporting `ohcrab`

In order for `ohcrab` to work in your terminal, you need to export the correct
function for your shell. Currently, we support `bash` and `zsh`.


## Usage

In the terminal, after typing the wrong command, type `crab` (or the alias you
chose in during the [Exporting `ohcrab`](#exporting-ohcrab) step). It will show
a menu to choose the correct command from.

## Road map

- [] Inform the user which shell type is being used when the `ohcrab` shell
  function is generated.
- [] Add support to user shell aliases.
