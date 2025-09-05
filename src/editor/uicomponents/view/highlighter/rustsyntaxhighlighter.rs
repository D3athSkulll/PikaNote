use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

const KEYWORDS: [&str; 52] = [
    //rust needs explicit type and size mention for constant arrays
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
    "macro_rules",
    "union",
];

const TYPES: [&str; 22] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "Option", "Result", "String", "str", "Vec", "HashMap",
];

const KNOWN_VALUES: [&str; 6] = ["Some", "None", "true", "false", "Ok", "Err"];

use super::{Annotation, AnnotationType, Line, SyntaxHighlighter};
use crate::prelude::*;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&mut self, idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        let mut iterator = line.split_word_bound_indices().peekable();
        //peekable turns iterator into something where peek() can be used besides next()
        //peek returns next item without advancing iterator and next returns next item and advances the iterator

        while let Some((start_idx, _))= iterator.next(){
            let remainder = &line[start_idx..];
            //instead of passing word, now pass the remaining entire string , so highlighting fxn can use as many items as necessary for annotation

            if let Some(mut annotation) = annotate_char(remainder)
                .or_else(|| annotate_lifetime_specifier(remainder))
                .or_else(|| annotate_number(remainder))
                .or_else(|| annotate_keyword(remainder))
                .or_else(|| annotate_type(remainder))
                .or_else(|| annotate_known_value(remainder))
                //chaining all highlighting functions together
                {
                    annotation.shift(start_idx);
                    //move annotation to right, so its index is relative to full string, not substring
                    result.push(annotation);
                    //skip over any subsequent word which is already annotated
                    while let Some(&(next_idx,_))=iterator.peek(){
                        //use peek to obtain next item 
                        if next_idx>=annotation.end{
                            break;
                            //if next item is after current annotation, then consume it regularly in surrounding while, to start highlighting next part
                            // this is done using peek
                        }
                        iterator.next();
                        //for any case where word is still part of previous annotation, we want to consume and discard next word.
                    } 
                };
                
        }
        self.highlights.insert(idx, result);
    }

   

    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}

// use a helper fxn for taking remaining string , annotation type to apply to next word, validator fxn f. if fxn returns true, we annotate the word,
//this is done by defining a closure and define the signature of F like done below  
 fn annotate_next_word<F>(
        string: &str,
        annotation_type: AnnotationType,
        validator:F,
    )->Option<Annotation>
    where
        F: Fn(&str)->bool,{
        if let Some(word)= string.split_word_bounds().next(){
            if validator(word){
                //only new thing in fn is calling validator fn which is pased as fn argument
                return Some(Annotation{
                    annotation_type,
                    start:0,
                    end: word.len(),
                });
            }
        }
        None
    }

fn annotate_number(string: &str)-> Option<Annotation>{
    //using the new helper fxn, the highlighter fxn based on validator fn makes work easy
    annotate_next_word(string, AnnotationType::Number, is_valid_number)
}

fn annotate_type(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::Type, is_type)
}

fn annotate_keyword(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::Keyword, is_keyword)
}

fn annotate_known_value(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::KnownValue, is_known_value)
}

fn annotate_char(string:&str)-> Option<Annotation>{
    
    let mut iter = string.split_word_bound_indices().peekable();
    //use peek
    if let Some((_,"\'"))= iter.next(){
        //ensure opening quote is handled
        if let Some((_,"\\")) = iter.peek(){
            iter.next();//skip escape character
        }
        iter.next();//skip untill closing quote
        if let Some((idx,"\'"))=iter.next(){
            return Some(Annotation{
                annotation_type: AnnotationType::Char,
                start:0,
                end: idx.saturating_add(1),//including the close quote in annotation
            });
        }
    }
    None
}

fn annotate_lifetime_specifier(string: &str)-> Option<Annotation>{
    let mut iter = string.split_word_bound_indices();
    if let Some((_,"\'"))=iter.next(){
        if let Some((idx,next_word))=iter.next(){
            return Some(Annotation{
                annotation_type: AnnotationType::LifeTimeSpecifier,
                start:0,
                end: idx.saturating_add(next_word.len())
            });
        }
    }
    None
}

fn is_valid_number(word: &str) -> bool {
    //new fn to validate number
    if word.is_empty() {
        return false;
    }

    if is_numeric_literal(word) {
        return true;
        //check if its numeric literal before going over whole word
    }

    let mut chars = word.chars();

    //checking first character
    if let Some(first_char) = chars.next() {
        if !first_char.is_ascii_digit() {
            return false; //number must start with digit
        }
    }

    let mut seen_dot = false;
    let mut seen_e = false;
    let mut prev_was_digit = true;

    //iterate over remaining char
    for char in chars {
        match char {
            '0'..='9' => {
                // this creates range of characters, = includes '9' as well, match works on a range too
                prev_was_digit = true;
            }
            '_' => {
                if !prev_was_digit {
                    return false;
                }
                prev_was_digit = false;
            }
            '.' => {
                if seen_dot || seen_e || !prev_was_digit {
                    return false; //disallow multiple dots , dots after e or dots not after digit
                }
                seen_dot = true;
                prev_was_digit = false;
            }
            'e' | 'E' => {
                if seen_e || !prev_was_digit {
                    return false; //disallow multiple e or e not after digit
                }
                seen_e = true;
                prev_was_digit = false;
            }
            _ => {
                return false; //invalid character
            }
        }
    }
    prev_was_digit //must end with a digit
}

fn is_numeric_literal(word: &str) -> bool {
    if word.len() < 3 {
        //for literal need a leading zero  , a suffix and atleast one digit
        return false;
    }
    let mut chars = word.chars();
    if chars.next() != Some('0') {
        //check the first character for a leading  0
        return false;
    }

    let base = match chars.next() {
        //check second character for proper base
        Some('b' | 'B') => 2,
        Some('o' | 'O') => 8,
        Some('x' | 'X') => 16,
        _ => return false,
    };

    chars.all(|char| char.is_digit(base))
    //.all returns true if passed closure is true for every entry in iterator and false otherwise
}

fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word)
}

fn is_type(word: &str) -> bool {
    TYPES.contains(&word)
}

fn is_known_value(word: &str) -> bool {
    KNOWN_VALUES.contains(&word)
}

