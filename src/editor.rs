use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod annotatedstring;
mod command;
mod uicomponents;
mod documentstatus;
mod line;
mod position;
mod size;
mod terminal;

use annotatedstring::{AnnotatedString,AnnotationType};
use uicomponents::{CommandBar, MessageBar, View, StatusBar, UIComponent};//contains the components as a whole
use documentstatus::DocumentStatus;
use line::Line;
use position::{Col,Row,Position};
use size::Size;
use terminal::Terminal;

use self::command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewLine,
    Move::{Up,Down,Left,Right},
    System::{Dismiss, Quit, Resize, Save, Search},



};

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 3; // hardcoding amt of times to press Ctrl+Q

#[derive(Eq, PartialEq, Default)]
enum PromptType{
    Search,
    Save,
    #[default]//derive default trait for only None variant of the enum PromptType
    None,
}

impl PromptType{
    fn is_none(&self)-> bool{
        //need to implement this as Option is not present
        *self == Self::None
    }
}
#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    title: String,
    message_bar: MessageBar,
    command_bar: CommandBar,
    prompt_type: PromptType,//used to steer in prompt type we are on
    terminal_size: Size,
    quit_times: u8,
}

impl Editor {
    //region: Struct Lifecycle
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?; // Setup terminal (alternate screen, raw mode)

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.handle_resize_command(size); // using default struct and calling resize on it to set up properly

        editor.update_message("HELP: Ctrl+F = Find | Ctrl+S = Save | Ctrl+Q = Quit");

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            debug_assert!(!file_name.is_empty());
            if editor.view.load(file_name).is_err() {
                 editor.update_message(&format!("ERR: Could not open file: {file_name}"));
            } // Load file into buffer if filename given
        }

        editor.refresh_status(); // ask to refresh status, this method is called in every rendering cycle too
        Ok(editor)
    }
    // end region
    //region : Event Loop
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
                       #[cfg(not(debug_assertions))]
                    {
                        let _ = err;
                    }
                }
            }
            self.refresh_status();//we have better method to refresh now
        }
    }
    
        fn refresh_screen(&mut self) {

        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        } //ensuring rendering is appropiate

        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        let _ = Terminal::hide_caret();
        //start adding ui elements from bottom

        if self.in_prompt(){
            self.command_bar.render(bottom_bar_row);
        }else{
            self.message_bar.render(bottom_bar_row);
        }


        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        } //if height is atleast 2 , render status bar too
        if self.terminal_size.height > 2 {
            self.view.render(0);
        } //if height is greater than 2 , render view

        let new_caret_pos = if self.in_prompt(){
            Position{
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        }else{
            self.view.caret_position()
        };//ensure that caret block is correctly placed , if no command bar present do the same as before by querying View
        // if command bar present calc correct pos based on column within command_bar and its posn on terminal
        debug_assert!(new_caret_pos.col <= self.terminal_size.width);
        debug_assert!(new_caret_pos.row <= self.terminal_size.height);


        let _ = Terminal::move_caret_to(new_caret_pos);

        let _ = Terminal::show_caret();
        let _ = Terminal::execute(); // flushes commands
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.file_name);
        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    } // check if title is changed , if it is write to terminal, update internal title to staty with terminal title
    
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
    //end region

    //region : Command Handling
    fn process_command(&mut self, command: Command) {
        if let System(Resize(size))= command{
            self.handle_resize_command(size);
            return;
        }
         // this block is there to correctly handle multiple quit events, or reset quit times, also resizing terminal shouldnt reset quit counter
    
        match self.prompt_type{
            PromptType::Search => self.process_command_during_search(command),
            PromptType::Save => self.process_command_during_save(command),
            PromptType::None => self.process_command_no_prompt(command),
        }
    }
    fn process_command_no_prompt(&mut self, command: Command){
        //handles commands outside prompt
        if matches!(command, System(Quit)){
            self.handle_quit_command();
            return;
        }
        self.reset_quit_times();//reset quit times for all other commands
        match command{
            System(Quit | Resize(_) | Dismiss)=>{}, // handled above
            System(Search)=>self.set_prompt(PromptType::Search),
            System(Save)=>self.handle_save_command(),
            Edit(edit_command)=>self.view.handle_edit_command(edit_command),
            Move(move_command)=>self.view.handle_move_command(move_command),
        }
    }
    //end region
    //region: Resize Command Handling
        fn handle_resize_command(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        let bar_size=Size{
            height:1,
            width: size.width,
        };
        self.message_bar.resize(bar_size);
        self.status_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
    } // defining the sizes and height of the ui
    //end region
    //region: Quit Command Handling
    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit_command(&mut self){
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES{
            self.should_quit=true;
        }else if self.view.get_status().is_modified{
            //handle the case where view is modified and user wants to do more work 
            self.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl+Q {} more times to quit. ",
                QUIT_TIMES - self.quit_times -1 
            ));
            self.quit_times += 1;
        }
    }

    fn reset_quit_times(&mut self){
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.update_message("");
            // if previously quit times was non zero, dispose the current message
        }
    }
    //end region

    //region: Save Command Handling
        fn handle_save_command(&mut self){
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }//calls save  or opens a prompt depending on status of file being loaded or not
    fn process_command_during_save(&mut self, command: Command){
        match command{
             System(Quit | Resize(_) | Search | Save) | Move(_) => {} // Not applicable during save, Resize already handled at this stage
             System(Dismiss)=>{
                self.set_prompt(PromptType::None);
                self.update_message("Save Aborted.");
             }
             Edit(InsertNewLine)=>{
                let file_name = self.command_bar.value();
                self.save(Some(&file_name));
                self.set_prompt(PromptType::None);
             }
             Edit(edit_command)=> self.command_bar.handle_edit_command(edit_command),
        }
    }
    fn save(&mut self, file_name: Option<&str>){
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if result.is_ok() {
            if self.view.save().is_ok(){
                self.update_message("File saved successfully.");
            } else{
                self.update_message("Error writing file!");
            }
        }
    }
    //end region

    //Region: Search Command & Prompt Handling
    fn process_command_during_search(&mut self, command: Command){
        match command{
            
            System(Dismiss) =>{
                self.set_prompt(PromptType::None);
                self.view.dismiss_search();//restore old text location and scroll to it
            }
            Edit(InsertNewLine)=>{
                self.set_prompt(PromptType::None);
                self.view.exit_search();//doesnot restore old textlocation and retains current posn in buffer
            }
            Edit(edit_command)=>{
                self.command_bar.handle_edit_command(edit_command);
                let query = self.command_bar.value();
                self.view.search(&query);//handle input and perform actual search
            }
            Move(Right | Down)=> self.view.search_next(),
            Move(Up | Left) => self.view.search_prev(),
            System(Quit| Resize(_)| Search | Save)| Move(_)=>{}
        }
    } 
    //end region 

    //region: Message and Command Bar
    fn update_message(&mut self, new_message: &str){
        self.message_bar.update_message(new_message);
    }
    // end region

    //region: Prompt Handling
    fn in_prompt(&self)->bool{
        !self.prompt_type.is_none()
    }

    fn set_prompt(&mut self, prompt_type: PromptType){
        //command now sets and enters apt prompt, based on prompt_type it sets  up the command bar clears value and sets internal prompt type and trigger redraw of message bar
        match prompt_type{
            PromptType::None=>self.message_bar.set_needs_redraw(true),//ensure message bar redraw properly in next cycle
            PromptType::Save=>self.command_bar.set_prompt("Save as: "),
            PromptType::Search=> {
                self.view.enter_search();
                self.command_bar
                    .set_prompt("Search (Esc to cancel, and Arrows to navigate) : ");
            }
        }
        self.command_bar.clear_value();
        self.prompt_type=prompt_type;
    }
    //end region

}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye. \r\n");
        }
    }
} // enables proper cleanup  regardless of panic or quit
