use std::io::Error;

use super::super::Size;

pub trait UIComponent {
    // Marks this UI component as in need of redrawing (or not)
    fn set_needs_redraw(&mut self, value: bool);
    // Determines if a component needs to be redrawn or not
    fn needs_redraw(&self) -> bool;

    // need to implement these methods in respective component

    fn resize(&mut self, size: Size) {
        self.set_size(size);
        self.set_needs_redraw(true);
    } // by this we can implement the fxn for all components by calling the trait

    //
    fn set_size(&mut self, size: Size);
    // method to update size, implement this too in respective component

    fn render(&mut self, origin_row: usize) {
        if self.needs_redraw(){
            if let Err(err) = self.draw(origin_row) {
            #[cfg(debug_assertions)]
            {
                panic!("Could not render component: {err:?}");
            }
            #[cfg(not(debug_assertions))]
            {
                let _ = err;
            }
        } else {
            self.set_needs_redraw(false);
        }//restructure to not use match and muted the clippy warning
        }
        
    } // method to draw this component if it is in need of redrawing

    fn draw(&mut self, origin_row: usize) -> Result<(), Error>;
    // method to actually draw component, needs to be implemeneted in component
} //new trait for view, messagebar, statusbar
