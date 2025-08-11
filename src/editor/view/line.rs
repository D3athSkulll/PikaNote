use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use std::{ fmt::{self},ops::Range};

#[derive(Clone, Copy)]
enum GraphemeWidth{
    Half,
    Full,
}

impl GraphemeWidth{
    const fn saturating_add(self, other: usize)->usize{
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
            
        }
    }
}

struct TextFragment{
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>

}

#[derive(Default)]
pub struct Line{
    fragments: Vec<TextFragment>,
}

impl Line{
    pub fn from(line_str: &str)-> Self{
        let fragments = Self::str_to_fragments(line_str);
        Self { fragments }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str.graphemes(true)//turn string to grapheme
            .map(|grapheme|{
                let (replacement, rendered_width)=Self::replacement_character(grapheme)
                    .map_or_else(||{
                        let unicode_width=grapheme.width();
                        let rendered_width = match unicode_width{
                            0|1=>GraphemeWidth::Half,
                            _=>GraphemeWidth::Full,
                        };
                        (None, rendered_width)
                    },
                    |replacement| (Some(replacement), GraphemeWidth::Half),
                );

                 TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                } // construct a text fragment
            })
            .collect()
        
       
        
    }

    fn replacement_character(for_str: &str)-> Option<char>{
        let width = for_str.width();
        match for_str{
            " "=> None,
            "\t"=> Some(' '),
            _ if width>0 && for_str.trim().is_empty()=>Some('␣'),
            _ if width ==0 =>{
                let mut chars= for_str.chars();
                if let Some(ch)=chars.next(){
                    if ch.is_control()&& chars.next().is_none(){
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _=>None,
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

        // Fully before visible range → skip
        if fragment_end <= range.start {
            current_pos = fragment_end;
            continue;
        }

        // Partially visible → show ellipsis
        if fragment_end > range.end || current_pos < range.start {
            result.push('⋯');
        }
        // Fully visible with replacement char
        else if let Some(char) = fragment.replacement {
            result.push(char);
        }
        // Fully visible, normal grapheme
        else {
            result.push_str(&fragment.grapheme);
        }

        current_pos = fragment_end;
    }

    result
}


    pub fn grapheme_count(&self)-> usize{
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize)-> usize{
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width{
                GraphemeWidth::Half=>1,
                GraphemeWidth::Full=>2,
            })
            .sum()
    }

    pub fn insert_char(&mut self, character: char, at: usize) {
    const TAB_WIDTH: usize = 5; // number of spaces per tab
    let mut result = String::new();

    for (index, fragment) in self.fragments.iter().enumerate() {
        if index == at {
            // if at place of insertion, push character to result string
            if character == '\t' {
                result.push_str(&" ".repeat(TAB_WIDTH)); // expand tab into spaces
            } else {
                result.push(character);
            }
        }
        result.push_str(&fragment.grapheme);
        if at >= self.fragments.len() {
            if character == '\t' {
                result.push_str(&" ".repeat(TAB_WIDTH)); // expand tab into spaces
            } else {
                result.push(character);
            }
        }//ensuring to push character even at end of line
    }
    self.fragments = Self::str_to_fragments(&result);// rebuild the structure
}

    pub fn delete(&mut self, at: usize){
        let mut result = String::new();
        for (index,fragment) in self.fragments.iter().enumerate(){
            if index != at{
                result.push_str(&fragment.grapheme);

            }
            
            
        }
        self.fragments=Self::str_to_fragments(&result);
    }



    pub fn append(&mut self, other: &Self){
        let mut concat = self.to_string();
        concat.push_str(&other.to_string());
        self.fragments=Self::str_to_fragments(&concat);

    }

    pub fn split(&mut self,at: usize)->Self{
        if at>self.fragments.len(){
            return Self::default();
        }
        let remainder = self.fragments.split_off(at);
        Self { fragments: remainder, }
    }
   
    
}

impl fmt::Display for Line{
    fn fmt(&self, formatter: &mut fmt::Formatter)-> fmt::Result{
        let result: String = self
            .fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect();
        write!(formatter, "{result}")
    }
}