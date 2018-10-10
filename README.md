# caves

This is a game written in the [Rust programming language][rust-lang] to
explicitly target the [GameShell], a small GameBoy sized Linux computer. The
game is a cave/dungeon exploration game. The maps are procedurally generated
based on a random seed that can be used to reproducibly create the same map on
multiple executions.

This game is still in very early development. Everything, including the name of
the game, is subject to change.

[rust-lang]: https://www.rust-lang.org
[GameShell]: (https://www.clockworkpi.com/)

## Running The Game

Visit [rustup.rs] to install the Rust compiler and Cargo.

Use `DISPLAY_SCALE=n` for some `n >= 1` to make seeing the game easier.

```bash
$ DISPLAY_SCALE=2 cargo run
```

[rustup.rs]: https://rustup.rs/
