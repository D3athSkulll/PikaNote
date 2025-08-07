use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod view;
use crossterm::event::{Event, read,KeyEvent, KeyEventKind};
use view::View;

mod editorcommand;
use editorcommand::EditorCommand;

mod terminal;
use terminal::Terminal;

pub struct Editor {
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
            view.load(file_name); // Load file into buffer if filename given
        }
        Ok(Self {
            should_quit: false,
            view,
        })
    }

    pub fn run(&mut self) {
        loop {
            let _ = self.refresh_screen(); // draw UI

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
            match EditorCommand::try_from(event) {
                Ok(command) => {
                    if matches!(command, EditorCommand::Quit) {
                        self.should_quit = true;
                    } else {
                        self.view.handle_command(command);
                    }
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not handle command : {err}");
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Recieved and discarded unsupported  or non-press event.");
            }
        }
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        let _ = Terminal::hide_caret()?;
        self.view.render(); // draws the file/buffer/render text
        let _ = Terminal::move_caret_to(self.view.get_position());

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
