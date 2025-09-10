#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
mod editor;
use editor::Editor;
mod logger;
use logger::Logger;
use log::{error,info};
mod prelude;

use crate::prelude::*;


fn main() {
    let logger = Logger::new("logs",{NAME});
    if let Err(e) = logger.init(){
        eprintln!("Failed to initialize {NAME}: {}", e);
        return;
    }

    info!("Logger initialized, Starting {NAME} editor... ");

    //create editor instance
    let mut editor = match Editor::new(){
        Ok(ed)=>{
            info!("Editor initialized successfully.");
            ed
        },
        Err(e)=>{
            error!("Failed to initialize Editor: {}",e);
            return;
        }
    };

    // Run editor
    if let Err(e) = editor.run() {
        error!("Editor terminated with error: {}", e);
    } else {
        info!("Editor exited successfully.");
    }

    info!("{NAME} editor closed.");

}
