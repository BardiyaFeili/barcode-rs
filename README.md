# Barcode

Barcode is a modal, terminal-based code editor written in Rust.

<img width="2400" height="1350" alt="image" src="https://github.com/user-attachments/assets/da63230d-42c4-4e44-958f-4acbaff85207" />

## The Basics

- **Modal Editing:** If you've used Vim, you'll feel at home. It’s got Normal, Insert, and Command modes.
- **Tree-sitter Support:** Fast and accurate syntax highlighting that actually understands your code's structure.
- **Split Windows:** Open multiple files and tile them however you want.
- **Lightweight:** It starts up instantly because it doesn't have 500 MB of telemetry attached.
- **Configurable:** Tweak your colors, line numbers, and behavior in `config.toml`.

## Getting Started

You'll need a working Rust toolchain. Then just:

```bash
git clone https://github.com/yourusername/barcode-rs
cd barcode-rs
cargo run --release
```

To open specific files, just pass them as arguments:

```bash
cargo run -- src/main.rs src/lib.rs
```

## How to use it (for now)

- `i`: Enter **Insert mode** to actually type things.
- `Esc`: Back to **Normal mode**.
- `:`: Enter **Command mode**.
  - `:w`: Save the current file.
  - `:q`: Quit the current buffer.
  - `:wq`: Save and quit.
  - `:n`: Create a new empty buffer.
  - `:e <path>`: Open a file.

Check out [CONFIG.md](CONFIG.md) to see Configuration options.
