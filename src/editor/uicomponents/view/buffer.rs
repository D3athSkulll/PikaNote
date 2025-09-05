use super::super::super::AnnotatedString;
use super::FileInfo;
use super::Highlighter;
use super::Line;
use super::Location;
use crate::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Error;
use std::io::Write;
use std::ops::Range;


#[derive(Default)]
pub struct Buffer {
     lines: Vec<Line>,
     dirty: bool,
     file_info: FileInfo,
}// cleaned up buffer defn to have better reasoning

impl Buffer {

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub const fn get_file_info(&self) -> &FileInfo {
        &self.file_info
    }

    pub fn grapheme_count(&self, idx: LineIdx) -> GraphemeIdx {
        self.lines.get(idx).map_or(0, Line::grapheme_count)
    }
    pub fn width_until(&self, idx: LineIdx, until: GraphemeIdx) -> GraphemeIdx {
        self.lines
            .get(idx)
            .map_or(0, |line| line.width_until(until))
    }
    //helper fxns prev calc within view

    pub fn get_highlighted_substring(
        &self,
        line_idx: LineIdx,
        range: Range<GraphemeIdx>,
        highlighter: &Highlighter,
    )->Option<AnnotatedString>{
        self.lines.get(line_idx).map(|line|{
            line.get_annotated_visible_substr(range, 
                Some(&highlighter.get_annotations(line_idx)))
        })
    }// attempt to retrieve correct highlighted strng. gets the annotation from highlighter and calls updated method in line

    pub fn highlight(&self, idx:LineIdx, highlighter: &mut Highlighter){
        if let Some(line) = self.lines.get(idx){
            highlighter.highlight(idx,line);
        }
    }//new fn to update highlighter

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();

        for value in contents.lines() {
            lines.push(Line::from(value));
        } // loading content of lines of text file into lines Vector

        Ok(Self {
            lines,
            file_info: FileInfo::from(file_name),
            dirty: false,
        })
    }

    pub fn search_forward(&self, query: &str, from: Location)-> Option<Location>{
        //revamped search method with some iterator logic 
        if query.is_empty(){
            return None;
        }
        let mut is_first = true; // flag to mark first search as it should be done different
        for(line_idx, line) in self
            .lines
            .iter()//create iterator over all lines
            .enumerate()//turns it into an iterator over pair (line_idx,line). this is needed to calc location on a match
            .cycle()//make iterator endless that is if reach end then wrap to start, forever
            .skip(from.line_idx)//from this skip all the linesbefore ones we're interested in
            .take(self.lines.len().saturating_add(1))//one more to search current line twice(from middle, and from start)
            //now we have all lines in iterator we need, current line, all till end of doc, start of doc, current line
            {
                let from_grapheme_idx = if is_first{
                    is_first = false;// for first search start searching from given grapheme_idx, for others start from left
                    from.grapheme_idx
                }else{
                    0
                };
                if let Some(grapheme_idx) = line.search_forward(query,from_grapheme_idx){
                    return Some(Location{
                        grapheme_idx,
                        line_idx,
                    });
                }
            }
            None
    }

    pub fn search_backward(&self,query: &str, from: Location)-> Option<Location>{
        if query.is_empty(){
            return None;
        }
        let mut is_first = true;
        for (line_idx,line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(self.lines.len().saturating_sub(from.line_idx).saturating_sub(1))
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_idx = if is_first{
                is_first=false;
                from.grapheme_idx
            }else{
                line.grapheme_count()
            };
            if let Some(grapheme_idx) = line.search_backward(query, from_grapheme_idx){
                return Some(Location{
                    grapheme_idx,
                    line_idx
                });
            }

        }
        None
    }

    pub fn save_to_file(&self, file_info: &FileInfo) -> Result<(), Error> {
         if let Some(file_path) = &file_info.get_path() {
            let mut file = File::create(file_path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }else{
            #[cfg(debug_assertions)]
            {
                panic!("Attempting to save with no file present");
            }
        }
        //removal of self.dirty is essential to keep this non mutable, as mutable version would not work for case when we want to save to an existing file
        Ok(())
    }

    pub fn save_as(&mut self, file_name: &str)-> Result<(),Error>{
        let file_info = FileInfo::from(file_name);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
    }
    pub fn height(&self) -> LineIdx {
        self.lines.len()
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
       debug_assert!(at.line_idx <= self.height());
       
        if at.line_idx == self.height() {
            self.lines.push(Line::from(&character.to_string()));
            self.dirty = true;
        }
        // add new line at edge of document
        else if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert_char(character, at.grapheme_idx);
            self.dirty = true;
        } // if in document middle let line handle the insertion
    }
    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx >= line.grapheme_count()
                && self.height() > at.line_idx.saturating_add(1)
            {
                // checking if we are at end of current line and next line exists
                let next_line = self.lines.remove(at.line_idx.saturating_add(1));
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_idx < line.grapheme_count() {
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].delete(at.grapheme_idx);
                self.dirty = true;
            }
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_idx == self.height() {
            self.lines.push(Line::default());
            self.dirty = true;
        }
        //if at end of doc insert new line
        else if let Some(line) = self.lines.get_mut(at.line_idx) {
            let new = line.split(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), new);
            self.dirty = true;
        } //if in mid of doc, split current line and add splitted to self.lines at proper index
    }
}
