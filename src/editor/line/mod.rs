use std::{
    fmt::{self, Display},
    ops::{Deref, Range},
};
mod graphemewidth;
mod textfragment;
use graphemewidth::GraphemeWidth;
use textfragment::TextFragment;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::{AnnotatedString, AnnotationType, Col};

type GraphemeIdx = usize;
type ByteIdx = usize;
type ColIdx = usize;

#[derive(Default, Clone)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String, //store entire string
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        debug_assert!(line_str.is_empty() || line_str.lines().count() == 1);
        let fragments = Self::str_to_fragments(line_str);
        Self {
            fragments,
            string: String::from(line_str),
        }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .grapheme_indices(true) //turn string to grapheme
            .map(|(byte_idx, grapheme)| {
                // turn grapheme in to tuple with two elements and then destructure them
                let (replacement, rendered_width) = Self::get_replacement_character(grapheme)
                    .map_or_else(
                        || {
                            let unicode_width = grapheme.width();
                            let rendered_width = match unicode_width {
                                0 | 1 => GraphemeWidth::Half,
                                _ => GraphemeWidth::Full,
                            };
                            (None, rendered_width)
                        },
                        |replacement| (Some(replacement), GraphemeWidth::Half),
                    );

                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                    start_byte_idx: byte_idx, // store byte idx
                } // construct a text fragment
            })
            .collect()
    }
    fn rebuild_fragments(&mut self) {
        //helper fxn to rebuild fragment and replace self.fragment as result
        self.fragments = Self::str_to_fragments(&self.string);
    }

    fn get_replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();
        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && for_str.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                let mut chars = for_str.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    pub fn get_visible_graphemes(&self, range: Range<ColIdx>) -> String {
        self.get_annotated_visible_substr(range, None, None)
            .to_string()
    }

    // Gets the annotated string in the given column index.
    // Note that the column index is not the same as the grapheme index:
    // A grapheme can have a width of 2 columns.
    // Parameters:
    // - range: The range of columns to get the annotated string from.
    // - query: The query to highlight in the annotated string.
    // - selected_match: The selected match to highlight in the annotated string. This is only applied if the query is not empty.

    pub fn get_annotated_visible_substr(
        &self,
        range: Range<ColIdx>, //visible part of string to print out
        query: Option<&str>,  //search result to highlight
        selected_match: Option<GraphemeIdx>, //query to be highlighted in special way
    ) -> AnnotatedString {
        if range.start >= range.end {
            return AnnotatedString::default();
        }
        //create new annotated string
        let mut result = AnnotatedString::from(&self.string);
        //start with full string and  remove and replaec things untill arrive at destination

        //Annotate it based on search results
        if let Some(query) = query {
            if !query.is_empty() {
                // nested ifs to check if valid query present and non empty
                self.find_all(query, 0..self.string.len()).iter().for_each(
                    |(start_byte_idx, grapheme_idx)| {
                        if let Some(selected_match) = selected_match {
                            if *grapheme_idx == selected_match {
                                //ifs to check  for selected match to be passed to fxn and selected match is same grapheme index than current found match
                                result.add_annotation(
                                    AnnotationType::SelectedMatch,
                                    *start_byte_idx,
                                    start_byte_idx.saturating_add(query.len()),
                                );
                                return;
                            }
                        }
                        //for this case annotate the string at area of match with selected match , return early from closure
                        result.add_annotation(
                            AnnotationType::Match,
                            *start_byte_idx,
                            start_byte_idx.saturating_add(query.len()),
                        );
                        //on reaching this code we do get a match, but not selected match . for this annotate the string but with different annotation type
                    },
                );
            }
        }
        //insert replacement characters and truncating if needed
        // perform this backward as byte indices become off in case a replacement char has different width than original char
        let mut fragment_start = self.width();
        //result string contains original , raw string as read , its too long with no esc chars but is now annotated

        for fragment in self.fragments.iter().rev() {
            //iterate over all fragments from back to front
            let fragment_end = fragment_start;
            //to go to left keep the new fragment end as old fragment start
            fragment_start =fragment_start.saturating_sub(fragment.rendered_width.into());
            //new fragment start = prev fragment start reduced by grapheme width of current fragment
            if fragment_start > range.end {
                continue; // no processing needed if we havent reach visible range yet
            }

            //clip right if fragment range partially visible
            if fragment_start < range.end && fragment_end > range.end {
                result.replace(fragment.start_byte_idx, self.string.len(), "⋯");
                continue;
            }
            //here we truncate to right, replace entire right of string which is not visible with ellipses
            else if fragment_start == fragment_end {
                //truncate to right if we reach end of visible range
                result.replace(fragment.start_byte_idx, self.string.len(), "");
                continue;
            }

            //fragment end at start of range: remove entire left side of string
            if fragment_end <= range.start {
                result.replace(
                    0,
                    fragment
                        .start_byte_idx
                        .saturating_add(fragment.grapheme.len()),
                    "",
                );
                break; //End processing since all remaining fragments will be invisible.
            } else if fragment_start < range.start && fragment_end > range.start {
                // Fragment overlaps with the start of range: Remove the left side of the string and add an ellipsis
                result.replace(
                    0,
                    fragment
                        .start_byte_idx
                        .saturating_add(fragment.grapheme.len()),
                    "⋯",
                );
                break; //End processing since all remaining fragments will be invisible
            }
            // Fragment is fully within range: Apply replacement characters if appropriate
            if fragment_start >= range.start && fragment_end <= range.end {
                if let Some(replacement) = fragment.replacement {
                    let start_byte_idx = fragment.start_byte_idx;
                    let end_byte_idx = start_byte_idx.saturating_add(fragment.grapheme.len());
                    result.replace(start_byte_idx, end_byte_idx, &replacement.to_string());
                }
            }
        }
        result
    }

    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> Col {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn width(&self) -> Col {
        self.width_until(self.grapheme_count())
    } //convenience method to simplify CommandBar implementation
      // Inserts a character into the line, or appends it at the end if at == grapheme_count + 1

    pub fn insert_char(&mut self, character: char, at: GraphemeIdx) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.start_byte_idx, character);
            // use convenience method provided by string to insert character at byte_idx
        } else {
            self.string.push(character);
        } //if no fragment found, character should be added at end
        self.rebuild_fragments(); // rebuild fragments to acccount for updated clusters and byte indices
    }

    pub fn append_char(&mut self, character: char) {
        self.insert_char(character, self.grapheme_count());
    }

    pub fn delete(&mut self, at: GraphemeIdx) {
        debug_assert!(at <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.start_byte_idx;
            let end = fragment
                .start_byte_idx
                .saturating_add(fragment.grapheme.len());
            self.string.drain(start..end); //removes substring from start to end
            self.rebuild_fragments();
        }
    }

    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        self.rebuild_fragments(); // update existing string and remove all fragments and rebuild
    }

    pub fn split(&mut self, at: GraphemeIdx) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start_byte_idx);
            self.rebuild_fragments();
            Self::from(&remainder)
        } else {
            Self::default()
        }
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> Option<GraphemeIdx> {
       if byte_idx > self.string.len() {
            return None;
        }
        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_idx >= byte_idx)
    }

    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        debug_assert!(grapheme_idx <= self.grapheme_count());
        if grapheme_idx == 0 || self.grapheme_count() == 0 {
            return 0;
        }
        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    panic!("Fragment not found for grapheme index: {grapheme_idx:?}");
                }
                #[cfg(not(debug_assertions))]
                {
                    0
                }
            },
            |fragment| fragment.start_byte_idx,
        )
    }

    pub fn search_forward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == self.grapheme_count() {
            return None;
        }
        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);
        self.find_all(query, start_byte_idx..self.string.len())
            .first()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }

    pub fn search_backward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        
        debug_assert!(from_grapheme_idx <= self.grapheme_count());

        if from_grapheme_idx == 0 {
            return None;
        }
        let end_byte_index = if from_grapheme_idx == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        }; //create a substring to search in. for backward search , go from beginning to current loc
        self.find_all(query, 0..end_byte_index)
            .last()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }

    fn find_all(&self, query: &str, range: Range<ByteIdx>)->Vec<(ByteIdx,GraphemeIdx)>{
        let end_byte_idx = range.end;
        let start_byte_idx = range.start;

        self.string
            .get(start_byte_idx..end_byte_idx)
            .map_or_else(Vec::new, |substr|{
                //get desired substring if not found return empty vector , if found return closure
                substr
                    .match_indices(query)// find potential matches within substring
                    .filter_map(|(relative_start_idx,_)|{
                        //call match indices and pass result to filter map where we filter unneeeded elements and map needed to some other structure by returning Some(New Structure) or None
                        let absolute_start_idx = relative_start_idx.saturating_add(start_byte_idx); //convert rel idx to abs idx
                         self.byte_idx_to_grapheme_idx(absolute_start_idx)
                            .map(|grapheme_idx| (absolute_start_idx, grapheme_idx))
                        })
                        .collect()

                    })
            
    }
}

impl Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.string)
    }
}

impl Deref for Line {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.string
    }
}
