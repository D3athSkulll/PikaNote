
use crossterm::cursor::MoveTo;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;

use crossterm::queue;
use crossterm::terminal::size;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::style::Print;
use std::io::{stdout,Write,Error};


#[derive(Copy,Clone)]
pub struct Size{
    pub width: u16,
    pub height: u16,
}
pub struct Position{
    pub x: u16,
    pub y: u16,
}
pub struct Terminal;

impl Terminal{

    pub fn initialize()-> Result<(), Error>{
        enable_raw_mode()?;
        Self::clear_screen();
        
        Self::move_cursor_to(Position{x:0,y:0})?;
        Ok(())
    }

    pub fn terminate()-> Result<(), Error>{
        disable_raw_mode()?;
        Ok(())
        
    }

    

    pub fn clear_screen()-> Result<(), Error>{
        
        queue!(stdout(), Clear(ClearType::All))?;
        queue!(stdout(), Clear(ClearType::Purge))?;   
        
        Ok(())
    }

    pub fn clear_current_line()-> Result<(), Error>{
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn hide_cursor()-> Result<(), Error>{
        queue!(stdout(), Hide)?;
        Ok(())
    }
    pub fn show_cursor()-> Result<(), Error>{
        queue!(stdout(), Show)?;
        Ok(())
    }

    pub fn move_cursor_to(position: Position) ->Result<(), Error>{
        queue!(stdout(), MoveTo(position.x,position.y))?;
        Ok(())
    }

    pub fn print(string: &str) -> Result<(),Error>
    {
        queue!(stdout(), Print(string))?;
        Ok(())
    }

    pub fn size() -> Result<Size, Error>{
        let (width, height) = size()?;
        Ok(Size{width, height})
    }

    pub fn execute() -> Result<(), Error>{
        stdout().flush()?;
        Ok(())
    }
}