# OhCrab! 🦀

`ohcrab` is a port of the well known CLI tool
[`thefuck`](https://github.com/nvbn/thefuck) to our beloved Rust language.

## Table of contents

1. [Installation](#installation)
1. [Usage](#usage)
1. [Road map](#road-map)
1. [Contributing](#contributing)

## Installation

### Prerequisite

For now, it is only possible to install `ohcrab` through `cargo`. If you don't
have cargo installed, you can installing it following the instructions from
[https://doc.rust-lang.org/cargo/getting-started/installation.html](https://doc.rust-lang.org/cargo/getting-started/installation.html).

### Installing the package

You can install `ohcrab` using [`cargo`](https://crates.io/):

```shell
cargo install ohcrab
```

### Exporting `ohcrab`

In order for `ohcrab` to work in your terminal, you need to export the correct
function for your shell. Currently, we support `bash` and `zsh`. Copy and paste
the respective command to your terminal:

* For `bash`, use:
```shell
eval $(ohcrab --shell bash)
```
* For `zsh`, use:
```shell
eval $(ohcrab --shell zsh)
```

### Loading `ohcrab` automatically

In order to load `ohcrab` every time you open a terminal, add the `eval`
command above to your .bash_profile, .bashrc, .zshrc or other startup script.

### Changing the alias

The commands above use the default alias (`crab`) to call `ohcrab` from your
terminal. Feel free to use your own alias by passing `--alias NEW_ALIAS` to use
your `NEW_ALIAS` instead. For example, in case you want to use `shinycrab` as
your alias in `zsh`, use

```shell
eval $(ohcrab --shell zsh --alias shinycrab)
```

## Usage

In the terminal, after typing the wrong command, type `crab` (or the alias you
chose in during the [Exporting `ohcrab`](#exporting-ohcrab) step). It will show
a menu to choose the correct command from.

## Road map

- [ ] Inform the user which shell type is being used when the `ohcrab` shell
  function is generated.
- [ ] Add support to user shell aliases.
- [ ] Add `sudo` support 
- [ ] Distribute binaries for Linux, MacOs and Windows.

## Contributing

If you like `ohcrab` and/or want to learn `rust`, you can contribute by adding
new rules or improving the crate.
