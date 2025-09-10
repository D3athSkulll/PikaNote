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
mod prelude;

use log::{error, info};

fn main() {
    // Initialize logging system
    logger::setup_logger().expect("Failed to set up logger");

    info!("Starting PikaNote editor...");

    // Construct editor
    let mut editor = match Editor::new() {
        Ok(ed) => ed,
        Err(e) => {
            error!("Failed to initialize editor: {}", e);
            return;
        }
    };

    match editor.run() {
        Ok(_) => info!("Editor exited successfully."),
        Err(e) => error!("Editor terminated with error: {}", e),
    }
}
