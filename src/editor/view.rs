use self::line::Line;
use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
    uicomponent::UIComponent,
    DocumentStatus, NAME, VERSION,
};
use std::{cmp::min, io::Error};

mod buffer;
mod line;
use buffer::Buffer;

#[derive(Clone, Copy, Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}
#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    // The view always starts at `(0/0)`. The `size` property determines the visible area.
    size: Size,

    text_location: Location,
    scroll_offset: Position,
}

impl View {
    //no need of new method as editor can create it from outside and configure as needed

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.height(),
            current_line_index: self.text_location.line_index,
            file_name: format!("{}", self.buffer.file_info), // use of debug trait for file info
            is_modified: self.buffer.dirty,
        }
    }

    //region:File io
    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.mark_redraw(true);
        }
    }

    fn save(&mut self) {
        let _ = self.buffer.save();
    } // saving modifies buffer by resetting dirty flag

    //end region
    // region: CommandHandling
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(_) | EditorCommand::Quit => {} // since resize is handled by editor, ignore quit and resize event
            EditorCommand::Move(direction) => self.move_text_location(direction),

            EditorCommand::Insert(character) => self.insert_char(character),
            EditorCommand::Delete => self.delete(),
            EditorCommand::Backspace => self.delete_backward(),
            EditorCommand::Enter => self.insert_newline(),
            EditorCommand::Save => self.save(),
        }
    }

    //endregion

    //region : Text editing\
    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.move_text_location(Direction::Right);
        self.mark_redraw(true);
    }

    fn delete_backward(&mut self) {
        if self.text_location.line_index != 0 || self.text_location.grapheme_index != 0 {
            self.move_text_location(Direction::Left);
            self.delete();
        }
    }
    fn delete(&mut self) {
        self.buffer.delete(self.text_location);
        self.mark_redraw(true);
    }

    fn insert_char(&mut self, character: char) {
        let old_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count); //get current grapheme width and set to 0 if no line present

        self.buffer.insert_char(character, self.text_location);

        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count); //do same thing again

        let grapheme_delta = new_len.saturating_sub(old_len);
        if grapheme_delta > 0 {
            //move right for added grapheme
            self.move_text_location(Direction::Right);
        }
        self.mark_redraw(true);
    }
    //issue tab is still detected as one blank space fix here and in line.rs

    //endregion

    // region: Rendering

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
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

    fn scroll_vertically(&mut self, to: usize) {
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
            self.mark_redraw(true);
        }
    }

    fn scroll_horizontally(&mut self, to: usize) {
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
            self.mark_redraw(true);
        }
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }
    //end region
    // region: Location and position handling

    pub fn caret_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = if let Some(line) = self.buffer.lines.get(row) {
            let gi = self.text_location.grapheme_index;
            let last = line.grapheme_count();

            if gi > 0 && gi == last {
                // End-of-line: place caret ON the last grapheme
                line.width_until(last - 1)
            } else {
                line.width_until(gi)
            }
        } else {
            0
        };

        Position { row, col }.saturating_sub(self.scroll_offset)
    }

    pub fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });
        Position { col, row }
    }
    //end region

    // region: text location movement

    fn move_text_location(&mut self, direction: Direction) {
        let Size { height, .. } = self.size;
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }

        self.scroll_text_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }
    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }
    #[allow(clippy::arithmetic_side_effects)]

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }
    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index = self.text_location.grapheme_index - 1;
        } else if self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }
    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }
    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
    }

    // Ensures self.location.grapheme_index points to a valid grapheme index by snapping it to the left most grapheme if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                min(line.grapheme_count(), self.text_location.grapheme_index)
            });
    }

    // Ensures self.location.line_index points to a valid line index by snapping it to the bottom most line if appropriate.
    // Doesn't trigger scrolling.

    fn snap_to_valid_line(&mut self) {
        let last_idx = self.buffer.height();
        self.text_location.line_index = min(self.text_location.line_index, last_idx);
    }
    //end region

    fn draw_symbol_fn() -> &'static str {
        "⚡"
    }
}

impl UIComponent for View {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let Size { height, width } = self.size;
        let end_y = origin_y.saturating_add(height);
        //allow this as we dont care welcome msg is put in perfect posn

        #[allow(clippy::integer_division)]
        let vertical_center = 2 * height / 3;
        let scroll_top = self.scroll_offset.row;
        for current_row in origin_y..end_y {
            //line_idx , take current(abs) row, subtract origin_y for row relative to view and add to scroll.offset

            let line_idx = current_row
                .saturating_sub(origin_y)
                .saturating_add(scroll_top);
            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                Self::render_line(current_row, &line.get_visible_graphemes(left..right))?;
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                let draw_symbol = Self::draw_symbol_fn();
                Self::render_line(current_row, draw_symbol)?;
            }
        }
        Ok(())
    }
}
