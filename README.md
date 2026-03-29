# picrossh

A terminal nonogram (picross) puzzle game, written in Rust using [ratatui](https://ratatui.rs/).

```
  ██████╗ ██╗ ██████╗██████╗  ██████╗ ███████╗███████╗██╗  ██╗
  ██╔══██╗██║██╔════╝██╔══██╗██╔═══██╗██╔════╝██╔════╝██║  ██║
  ██████╔╝██║██║     ██████╔╝██║   ██║███████╗███████╗███████║
  ██╔═══╝ ██║██║     ██╔══██╗██║   ██║╚════██║╚════██║██╔══██║
  ██║     ██║╚██████╗██║  ██║╚██████╔╝███████║███████║██║  ██║
  ╚═╝     ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝╚═╝  ╚═╝
```

## What is a nonogram?

Nonograms (also called picross or griddlers) are logic puzzles where you fill in cells on a grid according to number clues on the edges. Each clue describes the lengths of consecutive filled runs in that row or column. Every puzzle has a unique solution reachable by logic alone — no guessing required.

## Build & Run

**Requirements:** Rust toolchain (https://rustup.rs/)

```bash
# Run directly
cargo run

# Build release binary
cargo build --release
./target/release/picrossh

# Install to ~/.cargo/bin (makes `picrossh` available system-wide)
cargo install --path . --bin picrossh
```

## Controls

### Menu

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `Space` | Play selected puzzle |
| `q` | Quit |

### Playing

| Key | Action |
|-----|--------|
| `↑↓←→` / `hjkl` | Move cursor |
| `Shift+↑↓←→` / `HJKL` | Move cursor 5 cells |
| `g` | Jump to first row |
| `G` | Jump to last row |
| `0` | Jump to first column |
| `$` | Jump to last column |
| `Space` | Fill cell (hold + move to paint) |
| `e` / `E` | Erase cell (hold + move to erase) |
| `x` / `X` | Cross out cell |
| `r` | Reset board |
| `Esc` | Return to menu |
| `q` | Quit |

### After solving

| Key | Action |
|-----|--------|
| `n` | Next puzzle |
| `Esc` | Return to menu |
| `q` | Quit |

## Puzzles

Puzzles are grouped by size in the menu. Puzzle names are hidden until solved.

| Size | Puzzles |
|------|---------|
| 5×5  | Plus, Hourglass, Diamond, Steps, Z Shape |
| 10×10 | House, Heart, Arrow, Letter T, Ladder |
| 15×15 | Frame, Big Cross, H Letter |

All puzzles are verified to be uniquely solvable by constraint propagation (no guessing required).

## Export to NON format

Puzzles can be exported to the [NON format](http://www.lancaster.ac.uk/~simpsons/nonogram/fmt) for use with other nonogram tools:

```bash
cargo run --bin export_non
# writes puzzles/*.non
```

## License

MIT
