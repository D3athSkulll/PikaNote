use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod documentstatus;
mod editorcommand;
mod fileinfo;
mod statusbar;
mod terminal;
mod view;
use documentstatus::DocumentStatus;
use editorcommand::EditorCommand;
use statusbar::StatusBar;
use terminal::Terminal;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    title: String,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?; // Setup terminal (alternate screen, raw mode)

        let mut editor = Self {
            should_quit: false,
            view: View::new(2),
            status_bar: StatusBar::new(2),
            title: String::new(),
        }; // fill editor with default values, Issue: note that initially title is empty string

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            editor.view.load(file_name); // Load file into buffer if filename given
        }
        editor.refresh_status(); // ask to refresh status, this method is called in every rendering cycle too
        Ok(editor)
    }

    pub fn refresh_status(&mut self) {
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

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };
         if should_process {
            if let Ok(command) = EditorCommand::try_from(event) {
                if matches!(command, EditorCommand::Quit) {
                    self.should_quit = true;
                } else {
                    self.view.handle_command(command);
                    if let EditorCommand::Resize(size) = command {
                        self.status_bar.resize(size);
                } // if the terminal is resized the status bar should also be resized
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Recieved and discarded unsupported  or non-press event.");
            }
        }
    }
}

    fn refresh_screen(&mut self) -> Result<(), Error> {
        let _ = Terminal::hide_caret()?;
        self.view.render(); // draws the file/buffer/render text
        self.status_bar.render(); //draws the status bar
        let _ = Terminal::move_caret_to(self.view.caret_position());

        let _ = Terminal::show_caret()?;
        let _ = Terminal::execute()?; // flushes commands
        Ok(())
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
