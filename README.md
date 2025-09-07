# 📝 PikaNote

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)  
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)  
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)  
[![Code Style: Clippy](https://img.shields.io/badge/style-clippy-4e8cff?logo=rust)](https://github.com/rust-lang/rust-clippy)

**PikaNote** is a lightweight terminal-based text editor written in **Rust**.  
It emphasizes clean design, efficient terminal rendering, and a clear implementation of core editor functionalities.

> ⚠️ **Work in Progress** — actively developed with further features planned.

---

## Features

- **Keyboard Input & Event Handling**  
  Real-time processing of key events supporting character input, navigation, and control commands.

- **File Saving (`Ctrl + S`)**  
  Save the current buffer to disk, with overwrite capability.

- **Text Editing**  
  Supports insertion, deletion, and line breaks for basic editing.

- **File Viewing**  
  Open and navigate existing text files, including read-only modes.

- **Search (`Ctrl + F`)**  
  - Forward and backward search within the document  
  - Incremental (live) search with match highlighting  
  - Navigation between search matches

- **Annotated String System**  
  Internal metadata management allowing features such as search match highlighting.

- **Line Decoration**  
  Each line is prefixed with a visual marker (`⚡`) for UI clarity.

- **Terminal Rendering**  
  Efficient screen updates leveraging `crossterm::queue!` for minimal redraw overhead.

- **Status Bar**  
  Displays cursor position, file information, and editor mode.

- **Cursor Control**  
  Dynamic hiding and restoring of the terminal cursor for clean rendering.

- **Graceful Exit (`Ctrl + Q`)**  
  Exits the editor safely restoring terminal state.

- **Code Quality**  
  Enforced via `clippy` to ensure idiomatic, warning-free Rust code.

---

## Planned Enhancements

- Integration of [`ropey`](https://crates.io/crates/ropey) for efficient buffer management and large file support.

- Syntax highlighting powered by [`syntect`](https://github.com/trishume/syntect).

- Undo/Redo functionality to track and revert edits.

- Display of line numbers and gutter for improved navigation.

- Optional modal editing similar to Vim’s command mode.

---

## Getting Started

### Prerequisites

- Rust (latest stable version)  
  Install from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Build and Run

```bash
git clone https://github.com/D3athSkulll/PikaNote.git
cd PikaNote
cargo run
