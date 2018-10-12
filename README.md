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

## Story

A highly experienced explorer finds themselves deep within a cave full of
winding tunnels and rooms filled with items and many monsters ready to fight. At
the deepest levels of this cave is an enormous treasure. The explorer must fight
through the enemies and solve the puzzles of the cave in order to get to the
final treasure chamber at the deepest level of the cave.

Hidden in each level of the cave is a special golden "treasure key" which must
be collected in order to access the treasure chamber. The treasure chamber will
only be unlocked once all treasure keys are found and brought to the entrace of
the chamber.

Other keys can be used to unlock locked doors. Potions and other items are
scattered throughout to help the explorer fight through the cave. Solve the
puzzles of each level in order to reach the end and collect the treasure!

## Running The Game

Visit [rustup.rs] to install the Rust compiler and Cargo.

Use `DISPLAY_SCALE=n` for some `n >= 1` to make seeing the game easier on high
DPI displays.

```bash
$ DISPLAY_SCALE=2 cargo run
```

[rustup.rs]: https://rustup.rs/
