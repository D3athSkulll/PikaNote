use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use std::{ ops::Range};

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
pub struct Line{
    fragments: Vec<TextFragment>,
}

impl Line{
    pub fn from(line_str: &str)-> Self{
        let fragments= line_str
            .graphemes(true)//turn string to grapheme
            .map(|grapheme|{
                let unicode_width = grapheme.width();
                let rendered_width= match unicode_width{
                    0 | 1 => GraphemeWidth::Half,
                    _=> GraphemeWidth::Full,
                };// for each grapheme, determine unicode width and normalize to half or full
                let replacement = match unicode_width {
                    0 => Some('·'),
                    _ => None,
                };// replace 0 width character with middle dot
                 TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                } // construct a text fragment
            })
            .collect();
        Self{fragments} // returning the text fragment
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
   
    
}