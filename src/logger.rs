use crate::prelude::*;
use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::fs;

pub fn setup_logger()-> Result<(), fern::InitError>{
    //ensuring logging directory exists
    fs::create_dir_all("logs").ok();

    let log_file = format!("logs/editor-{}.log",Local::now().format("%Y-%m-%d"));

    Dispatch::new()
        //Global log level (changeable in RUST_LOG)
        .level(LevelFilter::Debug)
        //Respect RUST_LOG overrides
        .level_for({NAME},LevelFilter::Debug)
        .chain(std::io::stdout())//consolelogging
        .chain(fern::log_file(log_file)?)//File Logging
        .format(|out, message, record|{
            out.finish(format_args!(
                "{date} [{level}] [{target}] {message}",
                date = Local::now().format("%Y-%m-%d %H:%M:%S"),
                level = record.level(),
                target = record.target(),
                message = message,  
            ))
        })
        .apply()?;
    Ok(())
}