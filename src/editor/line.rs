use std::{fmt, ops::Range};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
    start_byte_idx: usize,// keep track of start byte index for grapheme
}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,//store entire string
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments = Self::str_to_fragments(line_str);
        Self { 
            fragments,
            string: String::from(line_str)
         }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .grapheme_indices(true) //turn string to grapheme
            .map(|(byte_idx, grapheme)| { // turn grapheme in to tuple with two elements and then destructure them
                let (replacement, rendered_width) = Self::get_replacement_character(grapheme)
                    .map_or_else(
                        || {
                            let unicode_width = grapheme.width();
                            let rendered_width = match unicode_width {
                                0 | 1 => GraphemeWidth::Half,
                                _ => GraphemeWidth::Full,
                            };
                            (None, rendered_width)
                        },
                        |replacement| (Some(replacement), GraphemeWidth::Half),
                    );

                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                    start_byte_idx: byte_idx // store byte idx
                } // construct a text fragment
            })
            .collect()
    }
    fn rebuild_fragments(&mut self){ 
        //helper fxn to rebuild fragment and replace self.fragment as result
        self.fragments=Self::str_to_fragments(&self.string);
    }

    fn get_replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();
        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && for_str.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                let mut chars = for_str.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }

        let mut result = String::new();
        let mut current_pos = 0; // holds amount of cells processed so far

        for fragment in &self.fragments {
            let fragment_end = fragment.rendered_width.saturating_add(current_pos);

            // If we've moved past the visible range, stop
            if current_pos >= range.end {
                break;
            }

            if fragment_end > range.start {
                if fragment_end > range.end || current_pos < range.start {
                    // Clip on the right or left
                    result.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }
            current_pos = fragment_end;
        }
        result
    }

    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }
    
    pub fn width(&self) -> usize {
        self.width_until(self.grapheme_count())
    }//convenience method to simplify CommandBar implementation
       // Inserts a character into the line, or appends it at the end if at > len of the string
    pub fn insert_char(&mut self, character: char, at: usize) {
        if let Some(fragment)= self.fragments.get(at){
            self.string.insert(fragment.start_byte_idx,character);
            // use convenience method provided by string to insert character at byte_idx
        }else{
            self.string.push(character);
        }//if no fragment found, character should be added at end
        self.rebuild_fragments(); // rebuild fragments to acccount for updated clusters and byte indices
    }
     pub fn append_char(&mut self, character: char) {
        self.insert_char(character, self.grapheme_count());
    }
    pub fn delete(&mut self, at: usize) {
        if let Some(fragment) = self.fragments.get(at) {

        let start = fragment.start_byte_idx;
            let end = fragment
                .start_byte_idx
                .saturating_add(fragment.grapheme.len());
             self.string.drain(start..end);//removes substring from start to end
             self.rebuild_fragments();
    }
}
    pub fn delete_last(&mut self){
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    pub fn append(&mut self, other: &Self) {
         self.string.push_str(&other.string);
        self.rebuild_fragments(); // update existing string and remove all fragments and rebuild
    }

    pub fn split(&mut self, at: usize) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start_byte_idx);
            self.rebuild_fragments();
            Self::from(&remainder)
        } else {
            Self::default()
        }
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: usize)-> usize{
        // conversion method
        for(grapheme_idx,fragment) in self.fragments.iter().enumerate(){
            if fragment.start_byte_idx >= byte_idx{
                return grapheme_idx;
            }
        }
        #[cfg(debug_assertions)]
        {
            panic!("Invalid byte_idx passed to byte_idx_to_grapheme_idx: {byte_idx:?}");
        }
        #[cfg(not(debug_assertions))]
        {
            0
        }

    }
    pub fn search(&self, query: &str)-> Option<usize>{
        self.string
            .find(query)// returns Option<usize> where usize is byte_idx
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx))
            //map converts one option to other
            //if find returns None, map returns None]
            // if find returns Some(x) then map works on that output to the closure

    }


}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.string)
    }
}
