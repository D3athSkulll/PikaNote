use std::{
    cmp::{max, min},
    fmt::{self, Display},
};
pub mod annotationtype;
pub use annotationtype::AnnotationType;
mod annotation;
use annotation::Annotation;
mod annotatedstringpart;
use annotatedstringpart::AnnotatedStringPart;
mod annotatedstringiterator;
use annotatedstringiterator::AnnotatedStringIterator;

#[derive(Default, Debug)]
pub struct AnnotatedString {
    string: String,
    annotations: Vec<Annotation>, //this ds holds our annotations serving as vehicle between Line to View to Terminal
}

impl AnnotatedString {
    pub fn from(string: &str) -> Self {
        Self {
            string: String::from(string),
            annotations: Vec::new(),
        }
    } // creates new unannotated string

    pub fn add_annotation(
        &mut self,
        annotation_type: AnnotationType,
        start_byte_idx: usize,
        end_byte_idx: usize,
    ) {
        debug_assert!(start_byte_idx <= end_byte_idx);
        self.annotations.push(Annotation {
            annotation_type,
            start_byte_idx,
            end_byte_idx,
        });
    } //this allows adding an annotation to the string

    pub fn replace(&mut self, start_byte_idx: usize, end_byte_idx: usize, new_string: &str) {
        //create a string based on full string and replace bits of it untill only what we need is left
        //this replace fn remains at core for this
        debug_assert!(start_byte_idx <= end_byte_idx);
        let end_byte_idx = min(end_byte_idx, self.string.len());
        //normalise the replacement untill the length of string in case someone attempts to replace more than what we have
        if start_byte_idx > end_byte_idx {
            return;
        }
        self.string
            .replace_range(start_byte_idx..end_byte_idx, new_string);
        //replaces our internal string
        // till now, some checks + replacing internal string

        let replaced_range_len = end_byte_idx.saturating_sub(start_byte_idx); //range we want to replace
        let shortened = new_string.len() < replaced_range_len;
        let len_difference = new_string.len().abs_diff(replaced_range_len); //how much shorterr or longer is the range

        if len_difference == 0 {
            //No adjustment of annotation needed as replacement has no length change
            return;
        } // no length difference , nothing to do

        self.annotations.iter_mut().for_each(|annotation| {
            //line gets a mutable reference to each annotation and performs below code, meaning items in self.annotations are changed without being copied or cloned
            annotation.start_byte_idx = if annotation.start_byte_idx >= end_byte_idx {
                //case 1: start of annotation beyond end of insertion point
                //here we need to move the annotation to right (if insert was bigger than removal) or to left if otherwise

                //for annotation starting after replaced range we move the start index by difference in length
                if shortened {
                    annotation.start_byte_idx.saturating_sub(len_difference)
                } else {
                    annotation.start_byte_idx.saturating_add(len_difference)
                }
            } else if annotation.start_byte_idx >= start_byte_idx {
                //case 2: start of annotation not beyound end of insertion but beyound start
                //starts in middle
                //here try to move the annotation to start accordingly , but at most to the boundaries
                //move start index by dif in len , constrained to beginning or end of replaced range

                if shortened {
                    max(
                        start_byte_idx,
                        annotation.start_byte_idx.saturating_sub(len_difference),
                    )
                } else {
                    min(
                        end_byte_idx,
                        annotation.start_byte_idx.saturating_add(len_difference),
                    )
                }
            } else {
                annotation.start_byte_idx
                //case 3 : start was before insertion point nothing to do
            };
            annotation.end_byte_idx = if annotation.end_byte_idx >= end_byte_idx {
                //for annotations ending after replaced range, we move the endindex by difference in length
                if shortened {
                    annotation.end_byte_idx.saturating_sub(len_difference)
                } else {
                    annotatin.end_byte_idx.saturating_add(len_difference)
                }
            } else if annotation.end_byte_idx >= start_byte_idx {
                // For annotations ending within the replaced range, we move the end index by the difference in length, constrained to the beginning or end of the replaced range.
                if shortened {
                    max(
                        start_byte_idx,
                        annotation.end_byte_idx.saturating_sub(len_difference),
                    )
                } else {
                    min(
                        end_byte_idx,
                        annotation.end_byte_idx.saturating_add(len_difference),
                    )
                }
            } else {
                annotation.end_byte_idx
            }//counter part for end of annotation
        });

        //filter out empty annotations, in  case previous step resulted in any
        self.annotations.retain(|annotation|{
            //retain removes all annotation for which closure false,
            //here remove any annotation which are empty i.e start==end or outside string
            annotation.start_byte_idx<annotation.end_byte_idx
                && annotation.start_byte_idx< self.string.len()
        });
    }
}


impl Display for AnnotatedString{
    fn fmt(&self, formatter: &mut fmt::Formatter)->fmt::Result{
        write!(formatter,"{}",self.string)
    }
}

impl <'a>IntoIterator for &'a AnnotatedString{
    //allows AnnotatedString to turn into iterator,
    //calling into_iter() on AnnotatedString will return AnnotatedStringIterator which returns AnnotatedStringPart s
    //lifetime ensure that no copy of string parts

    type Item =AnnotatedStringPart<'a>;
    type Iterator=AnnotatedStringIterator<'a>;

    fn into_iter(self)->Self::IntoIter{
        AnnotatedStringIterator{
        annotated_string: self,
        current_idx: 0 ,
        }
        
    }
}