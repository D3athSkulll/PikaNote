# 📝 PikaNote

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Code Style: Clippy](https://img.shields.io/badge/style-clippy-4e8cff?logo=rust)](https://github.com/rust-lang/rust-clippy)

**PikaNote** is a lightweight, terminal-based text editor written in **Rust**, focused on clean design, efficient terminal rendering, and a learning-first development approach.  
Inspired by [hecto](https://github.com/raphlinus/hecto), PikaNote implements core editor functionality with clarity and precision.

> ⚠️ **Work in Progress** — actively developed with more features on the way.

---

## ✨ Current Features

- **Keyboard Input & Event Handling**  
  Real-time key event processing with support for character entry, navigation, and control keys.

- **File Saving (`Ctrl + S`)** 💾  
  Save current buffer contents to a file with overwrite support.

- **Insert & Delete Text**  
  Basic editing functions like character insertion, backspace, and line breaks are implemented.

- **Text Viewer Capabilities**  
  Supports opening and navigating through existing text files without editing.

- **Search Functionality (`Ctrl + F` Trigger)** 🔍  
  - Forward search for a query string
  - Match highlighting using custom annotations 
  - Backward search
  - Incremental/live search
  - Scroll to next/previous match
  - 
- **Annotated String System** 🧵  
  Internal representation to attach metadata (like highlights) to the text buffer.

- **Exit Shortcut (`Ctrl + Q`)**  
  Gracefully exit the editor.

- **Line Decoration**  
  Adds a `⚡` symbol to every line as a placeholder UI element.

- **Clean Terminal Launch**  
  Clears the screen for a fresh start on editor launch.

- **Efficient Terminal Rendering**  
  Uses `crossterm::queue!` for low-overhead redrawing.

- **Status Bar**  
  Display cursor position, file info, and editing mode.
  
- **Cursor Control**  
  Dynamically hides the cursor during rendering for visual clarity.

- **Code Quality with Clippy**  
  Integrated with `clippy` to maintain idiomatic, warning-free Rust.

---

## 🔮 Upcoming Features

Planned improvements include advanced editing, performance boosts, and better UI/UX:

- **Efficient Buffer with `Ropey`**  
  Replace basic buffer with [`ropey`](https://crates.io/crates/ropey) for fast editing and large file support.

- **Syntax Highlighting with `syntect`**  
  Language-aware color highlighting using [`syntect`](https://github.com/trishume/syntect).

- **Undo/Redo Support**  
  Track editing history and support reverting changes.

- **Line Numbers & Gutter UI**  
  Display line numbers and improve visual layout.


- **Command Mode / Modal Editing**  
  Optional support for a Vim-style mode switcher.

---

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable version)  
  👉 [Install Rust](https://www.rust-lang.org/tools/install)

### Run Locally

```bash
git clone https://github.com/D3athSkulll/Text-Editor.git
cd Text-Editor
cargo run
