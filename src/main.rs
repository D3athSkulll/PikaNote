#![warn(clippy::all, clippy::pedantic)]// activate warning for clippy

mod editor;
use editor::Editor;

fn main(){
    let editor = Editor::default();
    editor.run();
}