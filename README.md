# tsumego-solver

A program for solving and generating tsumego puzzles, based on the paper [_Search versus Knowledge for Solving Life and Death Problems in Go_](https://www.aaai.org/Papers/AAAI/2005/AAAI05-218.pdf).

[Example puzzles](https://github.com/cameron-martin/tsumego-solver/releases)

## Installation

You can download the CLI from [github releases](https://github.com/cameron-martin/tsumego-solver/releases).

## Usage

### Generating puzzles

The following command will generate puzzles and output them to the `generated_puzzles` directory.

```sh
./cli generate
```

## Development

### Running benchmarks

```sh
RUSTFLAGS="-C target-cpu=native" cargo bench
```

### Generating asm

```sh
RUSTFLAGS="-g --emit asm -C target-cpu=native -Z asm-comments" cargo build --release
```

Note the [asm-comments flag](https://github.com/rust-lang/rust/pull/53290) only works with nightly rust.
