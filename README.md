# Barcode — a TUI Code Editor

Barcode is a simple, modal terminal code editor written in Rust.

This project is still very much a work in progress.

## Features

- Modal editing (Vim-inspired)
- Multiple tabs
- Fast and lightweight
- Cross-platform
- Built with extensibility in mind

## Why it’s minimal right now

Many features you would normally expect from an editor (syntax highlighting, plugins, LSP, etc.) are intentionally not implemented yet.

So far, the focus has been on:

- Getting the core architecture right
- Keeping the codebase simple and modular
- Making future features easy to add and fast to implement

This should allow new functionality to be added later with minimal refactoring.

## Building and running

Clone the repository and run:

    cargo build --release
    cargo run --release

## Usage

- Press `i` to enter Insert mode
- Other commands and modes are still under development

## Planned features

- Syntax highlighting
- Plugin system
- LSP support
- Configurable keybindings
