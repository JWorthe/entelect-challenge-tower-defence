# Rust Sample Bot

Rust is a systems programming language, giving programmers the low
level control that they would usually associate with a programming
langauge like C or C++, but modern high level programming features.

Rust is a compiled language, which compiles to an
architecture-specific binary.

For getting started with this bot in particular, I've done a write up
about [writing a Rust bot for the Entelect challenge](https://www.worthe-it.co.za/programming/2018/05/02/writing-an-entelect-challenge-bot-in-rust.html).

## Environment Setup

The Rust compiler toolchain can be downloaded from the Rust project
website.

https://www.rust-lang.org/en-US/install.html

## Compilation

The bot can be built using the Rust build tool, Cargo. For the sake of
the competition, the `--release` flag should be used.

```
cargo build --release
```

## Running

After compilation, there will be an executable in
`target/release/`.

For example, this sample bot's name is
`entelect_challenge_rust_sample`, so the executable to be run is
`target/release/entelect_challenge_rust_sample` on Linux or
`target/release/entelect_challenge_rust_sample.exe` on Windows.

