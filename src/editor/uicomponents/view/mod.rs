use super::super::{
    command::{Edit, Move},
    DocumentStatus, Line, Terminal,
};
use super::UIComponent;
use std::{cmp::min, io::Error};
use crate::editor::RowIdx;
use crate::prelude::*;

mod buffer;
use buffer::Buffer;
mod searchdirection;
use searchdirection::SearchDirection;
mod highlighter;
use highlighter::Highlighter;

mod fileinfo;
use fileinfo::FileInfo;
mod searchinfo;
use searchinfo::SearchInfo;

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    // The view always starts at `(0/0)`. The `size` property determines the visible area.
    size: Size,
    text_location: Location,
    scroll_offset: Position,
    search_info: Option<SearchInfo>,
}

impl View {
    //no need of new method as editor can create it from outside and configure as needed

    pub fn get_status(&self) -> DocumentStatus {
        let file_info = self.buffer.get_file_info();
        DocumentStatus {
            total_lines: self.buffer.height(),
            current_line_idx: self.text_location.line_idx,
            file_name: format!("{file_info}"), // use of debug trait for file info
            is_modified: self.buffer.is_dirty(),//Updates revolve around buffer grants no pub access to some fields 
            file_type: file_info.get_file_type(),//propagate file_type around Document_Statues
        }
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    } // allows editor to determine whether or not to prompt for file_name

    //region: Search
    pub fn enter_search(&mut self) {
        //entering means storing prev location
        self.search_info = Some(SearchInfo {
            prev_location: self.text_location,
            prev_scroll_offset: self.scroll_offset,
            query: None, //made query optional since no compulsion on query to be present while search active now
        });
    }

    pub fn exit_search(&mut self) {
        self.search_info = None;
        //exiting means dismissing the search_info and prev location
        self.set_needs_redraw(true); //we are now rendering  by highlighting search results, we need to explicitely request redraw upon exiting search , ensuring previously highlighted search results not highlighted
    }

    pub fn dismiss_search(&mut self) {
        //restore old text location , scrolling to it and dismiss search info
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.prev_location;
            self.scroll_offset = search_info.prev_scroll_offset;
            self.scroll_text_location_into_view(); // ensure prev location still visible even if terminal resize during search
                                                   /*
                                                   Suppose you have a wide terminal. You store the text location and scroll offset and enter search. You resize the screen and dismiss search.
                                                   What happened is that the previous scroll offset would have placed the text position out of view. This is fixed by scrolling the text location into view again here.
                                                    */
        }
        self.exit_search();
    }

    pub fn search(&mut self, query: &str) {
        if let Some(search_info) = &mut self.search_info {
            search_info.query = Some(Line::from(query));
        }
        self.search_in_direction(self.text_location, SearchDirection::default());
        //calls new method and searches in default direction i.e forward
    }

    // Attempts to get the current search query - for scenarios where the search query absolutely must be there.
    // Panics if not present in debug, or if search info is not present in debug
    // Returns None on release.
    fn get_search_query(&self) -> Option<&Line> {
        //showcase how to retrive a double option search_info is optiona and query is also a option

        let query = self
            .search_info
            .as_ref()
            .and_then(|search_info| search_info.query.as_ref());

        debug_assert!(
            query.is_some(),
            "Attempting to search with malformed search info present"
        );
        query
    }

    fn search_in_direction(&mut self, from: Location, direction: SearchDirection) {
        //renamed function
        if let Some(location) = self.get_search_query().and_then(|query| {
            //get location of next match by getting query
            if query.is_empty() {
                None
            } else if direction == SearchDirection::Forward {
                self.buffer.search_forward(query, from)
            } else {
                self.buffer.search_backward(query, from)
            } //calling the specialised search fxn
        }) {
            self.text_location = location;
            self.center_text_location(); //handling the result as before
        };
        self.set_needs_redraw(true); //to make highlighting show up we trigger redraw upon search
    }

    pub fn search_next(&mut self) {
        let step_right = self
            .get_search_query()
            .map_or(1, |query| min(query.grapheme_count(), 1));

        let location = Location {
            line_idx: self.text_location.line_idx,
            grapheme_idx: self.text_location.grapheme_idx.saturating_add(step_right),
            //start the new search behind current match
        };
        self.search_in_direction(location, SearchDirection::Forward);
    }

    pub fn search_prev(&mut self) {

        self.search_in_direction(self.text_location, SearchDirection::Backward);
    }

    //end region

    //region:File io
    pub fn load(&mut self, file_name: &str) -> Result<(), Error> {
        let buffer = Buffer::load(file_name)?;

        self.buffer = buffer;
        self.set_needs_redraw(true);
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.buffer.save_as(file_name)
    } //allows saving by file name

    //end region
    // region: CommandHandling
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(character) => self.insert_char(character),
            Edit::Delete => self.delete(),
            Edit::DeleteBackward => self.delete_backward(),
            Edit::InsertNewLine => self.insert_newline(),
        }
    }
    pub fn handle_move_command(&mut self, command: Move) {
        let Size { height, .. } = self.size;
        match command {
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::PageUp => self.move_up(height.saturating_sub(1)),
            Move::PageDown => self.move_down(height.saturating_sub(1)),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }

    //endregion

    //region : Text editing\
    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.handle_move_command(Move::Right);
        self.set_needs_redraw(true);
    }

    fn delete_backward(&mut self) {
        if self.text_location.line_idx != 0 || self.text_location.grapheme_idx != 0 {
            self.handle_move_command(Move::Left);
            self.delete();
        }
    }
    fn delete(&mut self) {
        self.buffer.delete(self.text_location);
        self.set_needs_redraw(true);
    }

    fn insert_char(&mut self, character: char) {
         let old_len = self.buffer.grapheme_count(self.text_location.line_idx);

        self.buffer.insert_char(character, self.text_location);

        let new_len = self.buffer.grapheme_count(self.text_location.line_idx);

        let grapheme_delta = new_len.saturating_sub(old_len);
        if grapheme_delta > 0 {
            //move right for added grapheme
            self.handle_move_command(Move::Right);
        }
        self.set_needs_redraw(true);
    }
    //issue tab is still detected as one blank space fix here and in line.rs

    //endregion

    // region: Rendering

    fn render_line(at: RowIdx, line_text: &str) -> Result<(), Error> {
        Terminal::print_row(at, line_text)
    }
    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        let draw_symbol = Self::draw_symbol_fn().to_string();
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        let remaining_width = width.saturating_sub(1);

        if remaining_width < len {
            return draw_symbol;
        }
        format!(
            "{:<1}{:^remaining_width$}",
            { draw_symbol },
            welcome_message
        )
    }

    // end region
    // region: scrolling

    fn scroll_vertically(&mut self, to: RowIdx) {
        let Size { height, .. } = self.size;

        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };
        if offset_changed {
            self.set_needs_redraw(true);
        }
    }

    fn scroll_horizontally(&mut self, to: ColIdx) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };
        if offset_changed {
            self.set_needs_redraw(true);
        }
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }
    fn center_text_location(&mut self) {
        let Size { height, width } = self.size;
        let Position { row, col } = self.text_location_to_position();
        let vertical_mid = height.div_ceil(2);
        let horizontal_mid = width.div_ceil(2);
        self.scroll_offset.row = row.saturating_sub(vertical_mid);
        self.scroll_offset.col = col.saturating_sub(horizontal_mid);
        self.set_needs_redraw(true);
    }
    //end region
    // region: Location and position handling

    pub fn caret_position(&self) -> Position {
         self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    pub fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_idx;
        debug_assert!(row.saturating_sub(1) <= self.buffer.height());
        let col = self
            .buffer
            .width_until(row, self.text_location.grapheme_idx);
        Position { col, row }
    }
    //end region

    // region: text location movement

    fn move_up(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }
    fn move_down(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }
    #[allow(clippy::arithmetic_side_effects)]

    fn move_right(&mut self) {
       let grapheme_count = self.buffer.grapheme_count(self.text_location.line_idx);
        if self.text_location.grapheme_idx < grapheme_count {
            self.text_location.grapheme_idx += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }
    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_idx > 0 {
            self.text_location.grapheme_idx = self.text_location.grapheme_idx - 1;
        } else if self.text_location.line_idx > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }
    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_idx = 0;
    }
    fn move_to_end_of_line(&mut self) {
           self.text_location.grapheme_idx = self.buffer.grapheme_count(self.text_location.line_idx);
    }

    // Ensures self.location.grapheme_index points to a valid grapheme index by snapping it to the left most grapheme if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_grapheme(&mut self) {
            self.text_location.grapheme_idx = min(
            self.text_location.grapheme_idx,
            self.buffer.grapheme_count(self.text_location.line_idx),
        
        );
    }

    // Ensures self.location.line_index points to a valid line index by snapping it to the bottom most line if appropriate.
    // Doesn't trigger scrolling.

    fn snap_to_valid_line(&mut self) {
        let last_idx = self.buffer.height();
        self.text_location.line_idx = min(self.text_location.line_idx, last_idx);
    }
    //end region

    fn draw_symbol_fn() -> &'static str {
        "⚡"
    }
}

impl UIComponent for View {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_row: RowIdx) -> Result<(), Error> {
        let Size { height, width } = self.size;
        let end_y = origin_row.saturating_add(height);
        //allow this as we dont care welcome msg is put in perfect posn

        #[allow(clippy::integer_division)]
        let bottom_third = 2 * height.div_ceil(3);
        let scroll_top = self.scroll_offset.row;

        let query = self
            .search_info
            .as_ref()
            .and_then(|search_info| search_info.query.as_deref());
        let selected_match= query.is_some().then_some(self.text_location);
        let mut highlighter= Highlighter::new(query, selected_match);

         for current_row in 0..end_y {
            self.buffer.highlight(current_row, &mut highlighter); 
            //highlight from the start of the document to the end of the visible area, to ensure all annotations are up to date.
        }

        for current_row in origin_row..end_y {
            //line_idx , take current(abs) row, subtract origin_row for row relative to view and add to scroll.offset

            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);
            let left = self.scroll_offset.col;
            let right = self.scroll_offset.col.saturating_add(width);
            if let Some(annotated_string) = 
                self.buffer
                    .get_highlighted_substring(line_idx, left..right, &highlighter)
                    {
                        Terminal::print_annotated_row(current_row, &annotated_string)?;
                    
            } else if current_row == bottom_third && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                let draw_symbol = Self::draw_symbol_fn();
                Self::render_line(current_row, draw_symbol)?;
            }
        }
        Ok(())
    }
}
