#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]// activate warning for clippy

mod editor;
use editor::Editor;

fn main(){
    Editor::default().run();
}