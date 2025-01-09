#![warn(clippy::all, clippy::pedantic)]// activate warning for clippy

mod editor;
use editor::Editor;

fn main(){
    Editor::default().run();
}