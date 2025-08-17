use std::cmp::min;
//iterator over annotedString which holds string and some annotation(color)
//iterate over it and return annoted string parts, turn it into iterator using into_iter

use super::{AnnotatedString, AnnotatedStringPart};

pub struct AnnotatedStringIterator{
    pub annotated_string: &'a AnnotatedString, //we wish to refer original annotedstring but not own it as we just try to look into it,
    //'a i.e lifetime concept helps here by giving access as long as iterator is alive
    pub current_idx: usize, // iterator keeps track of current_byte_idx already processed by annoted string
}

impl<'a> Iterator for AnnotatedStringIterator<'a>{
    type Item = AnnotatedStringPart<'a>;// any item the iterator produces lives as long as iterator itself

    fn next(&mut self)-> Option<Self::Item>{
        //iterator's next fxn returns next item untill it runs out of items, and returns None when it runs out
        if self.current_idx >= self.annotated_string.string.len(){
            //defining exit criteria
            return None;
        }
        // find current active annotation
        if let Some(annotation) = self // Annoted String contains a vector of Annotations having start and end,
            // we try to handle case where we are currently in an annotation
            .annotated_string
            .annotations
            .iter()
            .filter(|annotation|{
                //filter removes all annotation fir which closure does not return true
                annotation.start_byte_idx <= self.current_idx
                    && annotation.end_byte_idx > self.current_idx
                    //checking annotation is active, boundaries of annotation thus include byte_start_idx and exclude part at end_idx

            })
            .last()//for multiple overlapping annotations , take the last one, useful during syntax highlighting
            {
                let end_idx = min(annotation.end_byte_idx,
                self.annotated_string.string.len());
                let start_idx = self.current_idx;
                //sets start and end of string slice at which annotation ends
                self.current_idx=end_idx;
                //Advance current idx before returning, so call to next annotation returns next
                return Some(AnnotatedStringPart{
                    string: &self.annotated_string.string(start_idx..end_idx),
                    annotation_type: Some(annotation.annotation_type)
                });
                //return annotedstring part which is slice of annoted   
            
            }
            //find boundary of nearest annotation
            let mut end_idx = self.annotated_string.string.len();
            for annotation in &self.annotated_string.annotations{
                if annotation.start_byte_idx > self.current_idx && annotation.start_byte_idx<end_idx{
                    end_idx = annotation.start_byte_idx;
                }
            }
            let start_idx = self.current_idx;
            self.current_idx = end_idx;
            //advance current idx before returning 
            Some(AnnotatedStringPart{
                string: &self.annotated_string.string[start_idx..end_idx],
                annotation_type: None,
            })//return part with no active annotation
    }
}