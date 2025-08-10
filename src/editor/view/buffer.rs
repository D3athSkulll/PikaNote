use std::io::Error;

use super::line::Line;
use super::Location;
use std::fs::read_to_string;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line> 
}

impl Buffer {
    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();

        for value in contents.lines() {
            lines.push(Line::from(value));
        } // loading content of lines of text file into lines Vector

        Ok(Self { lines }) 
    }
    pub fn is_empty(&self) -> bool{
        self.lines.is_empty()
    }
     pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_char(&mut self, character:char, at:Location){
        if at.line_index > self.lines.len(){
            return;
        }// dont insert anything more than one line below doc
        if at.line_index ==self.lines.len(){
            self.lines.push(Line::from(&character.to_string()));
        }// add new line at edge of document
        else if let Some(line) = self.lines.get_mut(at.line_index){
            line.insert_char(character, at.grapheme_index);
        }// if in document middle let line handle the insertion
    }
}