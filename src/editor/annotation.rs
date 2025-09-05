use crate::prelude::*;
use super::AnnotationType;
#[derive(Copy,Clone,Debug)]
#[allow(clippy::struct_field_names)]//naming field type is disallowed due to type being keyword
pub struct Annotation{
    pub annotation_type: AnnotationType,
    pub start: ByteIdx,
    pub end: ByteIdx,
   
}
//represents annotation on AnnotatedString

impl Annotation{
    pub fn shift(&mut self, offset: ByteIdx){
        //fn to     move annotation to right on a string . By this the highlighting functions will start to return annotation relative to substring passed
        self.start = self.start.saturating_add(offset);
        self.end = self.end.saturating_add(offset);
    }
}