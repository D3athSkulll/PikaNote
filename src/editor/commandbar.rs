use std::{cmp::min, io::Error, process::Command};

use super::{command::Edit,Line,Size,Terminal, UIComponent};

#[derive(Default)]
pub struct CommandBar{
    prompt: String,
    value: Line, // Line helps to reuse logic in handling wide characters in filenames
    needs_redraw: bool,
    size: Size,
}//new commandbar component for prompts

impl CommandBar{
    pub fn handle_edit_command(&mut self, command: Edit){
        match command{
            Edit::Insert(character)=>self.value.append_char(character),
            Edit::Delete | Edit::InsertNewLine =>{}
            Edit::DeleteBackward=> self.value.delete_last(),
        }
        self.set_needs_redraw(true);
    }

    pub fn caret_position_col(&self)-> usize{
        let max_width = self
            .prompt
            .len()
            .saturating_add(self.value.grapheme_count());
        min(max_width, self.size.width)
        //caret's x posn is either width of input + len of prompt (considering ASCII Character)
        // or its width of terminal ,  whichever is smaller
    }

    pub fn value(&self)->String{
        self.value.to_string()
    }

    pub fn set_prompt(&mut self, prompt: &str){
        self.prompt = prompt.to_string();
    }

}

impl UIComponent for CommandBar{
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: super::Size) {
        self.size = size;
    }

    fn draw(&mut self, origin: usize) -> Result<(), Error> {
        let area_for_value = self.size.width.saturating_sub(self.prompt.len());
        //space between right side of terminal and edge of bar
        let value_end = self.value.width();
        //prefer to show left part of value, so end of visible range we try to access = full width

        let value_start = value_end.saturating_sub(area_for_value);
        //gives start of grapheme subrange we want to print.

        let message = format!(
            "{}{}",
            self.prompt,
            self.value.get_visible_graphemes(value_start..value_end)
        );
        let to_print = if message.len()<=self.size.width{
            //if cant fit what needs to print then dont print anything
            message
        }else{
            String::new()
        };
        Terminal::print_row(origin, &to_print)
    }
}