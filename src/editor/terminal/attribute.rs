use crossterm::style::Color;

use super::super::AnnotationType;

pub struct Attribute{
    pub foreground: Option<Color>,
    pub background: Option<Color>,

}// struct defines attribute to be used by terminal. They define the styling to be applied to part of text , like bold, italics, underline
//here limit to color

impl From<AnnotationType> for Attribute{
    fn from(annotation_type: AnnotationType)->Self{
        //allows conversion of annotation type to attribute, seperating concerns , this will also map string highlights to specific colors
        match annotation_type{
            AnnotationType::Match=>Self{
                foreground: Some(Color::Rgb{
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                background: Some(Color::Rgb{
                    r:211,
                    g:211,
                    b:211,
                }),
            },
            AnnotationType::SelectedMatch=>Self{
                foreground: Some(Color::Rgb{
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                background: Some(Color::Rgb{
                    r:100,
                    g:255,
                    b:153,
                }),
            },
            AnnotationType::Number => Self {
                foreground: Some(Color::Rgb {
                    r: 255,
                    g: 99,
                    b: 71,
                }),
                background: None,
        
    }
}
    }
}