# PikaNote

A lightweight, terminal-based text editor built from scratch in **Rust** — inspired by the design philosophy of [hecto](https://github.com/raphlinus/hecto). This project is a learning-focused implementation of core editor functionality with a clean and efficient approach to terminal rendering.

> Currently a **work in progress** — actively being developed and expanded.

---

## Current Features

- **Keypress Event Handling**  
  Captures and processes real-time user input via keyboard.

- **Exit Shortcut (`Ctrl + Q`)**  
  Gracefully exits the editor using a standard shortcut.

- **Clean Terminal on Launch**  
  Automatically clears the terminal screen on startup for a fresh session.

- **Line Decoration**  
  Displays a `⚡` symbol at the beginning of each line for a distinguished layout.

- **Cursor Hiding**  
  Cursor is hidden during runtime using terminal control for a focused UI.

- **Efficient Screen Drawing**  
  Utilizes `crossterm::queue!` macro to optimize terminal rendering.

- **Static Analysis with Clippy**  
  Integrated with `clippy` to maintain idiomatic and error-free Rust code.

---

## Upcoming Features

Planned features will follow the development roadmap outlined in [hecto's documentation](https://github.com/raphlinus/hecto), including:

- File I/O (open/save)
- Editable buffer
- Cursor navigation
- Syntax highlighting
- Text search
- Undo/redo functionality

---

## Getting Started

### Prerequisites

- Rust (latest stable version)  
  [Install Rust](https://www.rust-lang.org/tools/install)

### Run Locally

```bash
git clone https://github.com/D3athSkulll/Text-Editor.git
cd Text-Editor
cargo run
