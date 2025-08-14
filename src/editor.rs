use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod command;
mod commandbar;
mod documentstatus;
mod line;
mod messagebar;
mod position;
mod size;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use commandbar::CommandBar;
use documentstatus::DocumentStatus;
use line::Line;
use messagebar::MessageBar;
use position::Position;
use size::Size;
use statusbar::StatusBar;
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

use self::command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewLine,
    System::{Dismiss, Quit, Resize, Save},



};

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 3; // hardcoding amt of times to press Ctrl+Q

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    title: String,
    message_bar: MessageBar,
    command_bar: Option<CommandBar>,
    terminal_size: Size,
    quit_times: u8,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?; // Setup terminal (alternate screen, raw mode)

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size); // using default struct and calling resize on it to set up properly

        editor
            .message_bar
            .update_message(&"HELP: Ctrl+S = Save | Ctrl+Q = Quit".to_string());

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            if editor.view.load(file_name).is_err() {
                editor
                    .message_bar
                    .update_message(&format!("ERR: Could not open file: {file_name}"));
            } // Load file into buffer if filename given
        }

        editor.refresh_status(); // ask to refresh status, this method is called in every rendering cycle too
        Ok(editor)
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        self.message_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        if let Some(command_bar) = &mut self.command_bar{
            command_bar.resize(Size{
                height:1,
                width: size.width,
            });//if command bar is active it needs to resize
        }
    } // defining the sizes and height of the ui

    fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.file_name);
        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    } // check if title is changed , if it is write to terminal, update internal title to staty with terminal title

    pub fn run(&mut self) {
        loop {
            self.refresh_screen(); // draw UI

            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => self.evaluate_event(event), // listen to keyboard or screen resize events
                error => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {error:?}");
                    }
                }
            }
            let status = self.view.get_status();
            self.status_bar.update_status(status);
        }
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };
        if should_process {
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command); // logic to work on a command is handled in othe fxn
            }
        }
    }
    fn process_command(&mut self, command: Command) {
        match command {
             System(Quit) => {
                if self.command_bar.is_none() {
                    self.handle_quit();
                }
            }//Only treat Quit when outside prompt
            System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(), //reset quit time for all other commands
        } // this block is there to correctly handle multiple quit events, or reset quit times, also resizing terminal shouldnt reset quit counter
    
        match command{
            System(Quit | Resize(_))=>{}, // handled above
            System(Save)=> {
                if self.command_bar.is_none(){
                    self.handle_save();
                }// we save only when outside of a prompt
            },
            System(Dismiss)=>{
                if self.command_bar.is_some(){
                    self.dismiss_prompt();
                    self.message_bar.update_message("Save aborted ");
                }
            },// Dismiss represents press on Esc, dismiss command bar for save aborted on Esc
            Edit(edit_command)=>{
                if let Some(command_bar) = &mut self.command_bar{
                    if matches!(edit_command, InsertNewLine){
                        let file_name = command_bar.value();
                        self.dismiss_prompt();
                        self.save(Some(&file_name));
                    }// get entered file name and attempt to save it and dismiss the prompt
                    else
                    {
                        command_bar.handle_edit_command(edit_command);
                    }//if we have a edit command and command bar is active , then fwd edit command to it
                }else{
                    self.view.handle_edit_command(edit_command);
                }//outside of command prompt, forward edit command to view
            },
            Move(move_command)=>{
                if self.command_bar.is_none(){
                    self.view.handle_move_command(move_command);
                }//move command forwarded to view if command bar inactive
            }

        }
    }

    fn dismiss_prompt(&mut self){
        self.command_bar= None;
        self.message_bar.set_needs_redraw(true);
    }// set command bar as none and make sure message bar redraws in next redraw cycle

    fn show_prompt(&mut self){
        let mut command_bar = CommandBar::default();
        command_bar.set_prompt("Save as : ");
        command_bar.resize(Size{
            height:1,
            width: self.terminal_size.width,
        });
        command_bar.set_needs_redraw(true);
        self.command_bar=Some(command_bar);
    }//showing prompt means create new command bar, set the text determinng what we prompt for , resize it to display it, set it to needs redraw

    fn handle_save(&mut self){
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.show_prompt();
        }
    }//calls save  or opens a prompt depending on status of file being loaded or not
    fn save(&mut self, file_name: Option<&str>){
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if result.is_ok() {
        if self.view.save().is_ok(){
            self.message_bar.update_message("File saved successfully.");
        } else{
            self.message_bar.update_message("Error writing file!");
        }
    }
} 
    

    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit(&mut self){
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES{
            self.should_quit=true;
        }else if self.view.get_status().is_modified{
            //handle the case where view is modified and user wants to do more work 
            self.message_bar.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl+Q {} more times to quit. ",
                QUIT_TIMES - self.quit_times -1 
            ));
            self.quit_times += 1;
        }
    }

    fn reset_quit_times(&mut self){
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.message_bar.update_message("");
            // if previously quit times was non zero, dispose the current message
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        } //ensuring rendering is appropiate
         let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        let _ = Terminal::hide_caret();
        //start adding ui elements from bottom
        if let Some(command_bar)= &mut self.command_bar{
            command_bar.render(bottom_bar_row);
        }else{
            self.message_bar.render(bottom_bar_row);
        } // deals with rendering of command or message bar

        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        } //if height is atleast 2 , render status bar too
        if self.terminal_size.height > 2 {
            self.view.render(0);
        } //if height is greater than 2 , render view

        let new_caret_pos = if let Some(command_bar)=&self.command_bar{
            Position{
                row: bottom_bar_row,
                col: command_bar.caret_position_col(),
            }
        }else{
            self.view.caret_position()
        };//ensure that caret block is correctly placed , if no command bar present do the same as before by querying View
        // if command bar present calc correct pos based on column within command_bar and its posn on terminal

        let _ = Terminal::move_caret_to(new_caret_pos);

        let _ = Terminal::show_caret();
        let _ = Terminal::execute(); // flushes commands
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye. \r\n");
        }
    }
} // enables proper cleanup  regardless of panic or quit
