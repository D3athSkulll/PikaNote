use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use super::{Annotation, AnnotationType, Line, SyntaxHighlighter};
use crate::prelude::*;

#[derive(Default)]
pub struct RustSyntaxHighlighter{
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl SyntaxHighlighter for RustSyntaxHighlighter{
    fn highlight(&mut self, idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        for(start_idx,word) in line.split_word_bound_indices(){
            if is_valid_number(word){
                //only issue that if multiple digit come in action then instead of saving single annotation for each, we save as group 
                result.push(Annotation{
                    annotation_type: AnnotationType::Number,
                    start: start_idx,
                    end: start_idx.saturating_add(word.len()),
                });
            }
        }
        self.highlights.insert(idx,result);
    }

    fn get_annotations(&self, idx:LineIdx)->Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}

fn is_valid_number(word: &str)->bool{
    //new fn to validate number
    if word.is_empty(){
        return false;
    }

    if is_numeric_literal(word){
        return true;
        //check if its numeric literal before going over whole word
    }

    let mut chars = word.chars();

    //checking first character
    if let Some(first_char) = chars.next(){
        if !first_char.is_ascii_digit(){
            return false;//number must start with digit 
        }
    }

    let mut seen_dot = false;
    let mut seen_e= false;
    let mut prev_was_digit = true;

    //iterate over remaining char
    for char in chars{
        match char{
            '0'..='9' =>{
                // this creates range of characters, = includes '9' as well, match works on a range too
                prev_was_digit=true;
            }
            '_'=>{
                if !prev_was_digit{
                    return false;    
                }
                prev_was_digit=false;
            }
            '.'=>{
                if seen_dot || seen_e || !prev_was_digit{
                    return false;//disallow multiple dots , dots after e or dots not after digit
                }
                seen_dot=true;
                prev_was_digit=false;
            }
            'e'|'E' =>{
                if seen_e || !prev_was_digit{
                    return false;//disallow multiple e or e not after digit
                }
                seen_e = true;
                prev_was_digit=false;   
            }
            _=>{
                return false;//invalid character
            }

        }
    }
    prev_was_digit//must end with a digit
}

fn is_numeric_literal(word: &str)->bool{
    if word.len()<3 {
        //for literal need a leading zero  , a suffix and atleast one digit
        return false;
    }
    let mut chars = word.chars();
    if chars.next() != Some('0'){
        //check the first character for a leading  0
        return false;
    }

    let base = match chars.next(){
        //check second character for proper base
        Some('b'|'B') =>2,
        Some('o'|'O') =>8,
        Some('x'|'X') =>16,
        _=> return false,
    };

    chars.all(|char| char.is_digit(base))
    //.all returns true if passed closure is true for every entry in iterator and false otherwise
}
