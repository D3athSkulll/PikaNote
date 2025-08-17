use super::AnnotationType;
#[derive(Debug)]
pub struct AnnotatedStringPart<'a>{
    pub string: &'a str,
    pub annotation_type: Option<AnnotationType>,
}// using lifetime specifiers necessary since creating a full blown copy is unnecesasary, just return a pointer to original string 
