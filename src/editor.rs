
use core::cmp::min;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod view;
use view::View;


mod terminal;
use terminal::{Position, Size, Terminal};


#[derive(Copy, Clone, Default)]
struct Location{
    x: usize,
    y: usize,
}



pub struct Editor {
    location: Location,
    should_quit: bool,
    view: View,
}

impl Editor {
    
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?; // Setup terminal (alternate screen, raw mode)

        let mut view = View::default(); // Create default View (which includes buffer)

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            view.load(file_name);// Load file into buffer if filename given 
        }
        Ok(Self {
            should_quit: false,
            location: Location::default(), // cursor at (0,0)
            view,
        })
    }

   pub  fn run(&mut self) {
        loop{
            let _ = self.refresh_screen(); // draw UI

            if self.should_quit{
                break;
            }

            match read(){
                Ok(event)=>self.evaluate_event(event), // listen to keyboard or screen resize events
                error   =>{
                    #[cfg(debug_assertions)]{
                        panic!("Could not read event: {error:?}" );
                    }
                }
            }
        }
        
    }
    fn move_point(&mut self, key_code: KeyCode) {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size().unwrap_or_default();
        match key_code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = min(height.saturating_sub(1), y.saturating_add(1));
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Right => {
                x = min(width.saturating_sub(1), x.saturating_add(1));
            }
            KeyCode::PageUp => {
                y = 0;
            }
            KeyCode::PageDown => {
                y = height.saturating_sub(1);
            }
            KeyCode::Home => {
                x = 0;
            }
            KeyCode::End => {
                x = width.saturating_sub(1);
            }
            _ => (),
        }
        self.location = Location { x, y }; // update location of cursor to new location as per special key press
        
    }

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        match event{
            Event::Key(KeyEvent{
                code,
                kind: KeyEventKind::Press,
                modifiers,
                ..
            })=>match(code,modifiers){
                (KeyCode::Char('q'),KeyModifiers::CONTROL)=>{
                    self.should_quit=true;
                }
                (
                    KeyCode::Up
                    |KeyCode::Down
                    |KeyCode::Left
                    |KeyCode::Right
                    |KeyCode::PageDown
                    |KeyCode::PageUp
                    |KeyCode::Home
                    |KeyCode::End,
                    _,
                )=>{
                    self.move_point(code); // move the cursor according to keypress
                }
                _=>{}
            },
            Event::Resize(width_u16,height_u16 )=>{
                #[allow(clippy::as_conversations)]
                let height = height_u16 as usize;

                #[allow(clippy::as_conversations)]
                let width = width_u16 as usize;
            
                self.view.resize(Size { height, width }); // resize the ui and text according to new Size parameters
            }
            _=>{}
        }
        
    }
        
      
    fn refresh_screen(&mut self) -> Result<(), Error> {
        let _ = Terminal::hide_caret()?;
        self.view.render();// draws the file/buffer/render text
        let _ = Terminal::move_caret_to(Position{
            col: self.location.x,
            row: self.location.y,
        });
        
        
        let _ = Terminal::show_caret()?;
        let _ =Terminal::execute()?; // flushes commands
        Ok(())
    }
    }

    impl Drop for Editor{
        fn drop(&mut self){
            let _ = Terminal::terminate();
            if self.should_quit{
                let _ = Terminal::print("Goodbye. \r\n");
            }
        }
    }// enables proper cleanup  regardless of panic or quit