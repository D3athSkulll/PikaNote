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