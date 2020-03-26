# tsumego-solver

A program for solving and generating tsumego puzzles, based on the paper [_Search versus Knowledge for Solving Life and Death Problems in Go_](https://www.aaai.org/Papers/AAAI/2005/AAAI05-218.pdf).

## Generating puzzles

```sh
cargo run --release generate
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
