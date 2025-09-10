use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::path::PathBuf;
use std::io;
use std::fs;

pub struct Logger{
    log_dir: PathBuf,
    base_name: String,
    current_date: String,  //YYYY-MM-DD
}

impl Logger{
    pub fn new(log_dir: impl Into<PathBuf>, base_name: impl Into<String>) -> Self {
        let now = Local::now();
        Self {
            log_dir: log_dir.into(),
            base_name: base_name.into(),
            current_date: now.format("%Y-%m-%d").to_string(),
        }
    }
    fn log_file_path(&self) -> PathBuf {
        self.log_dir.join(format!("{}-{}.log", self.base_name, self.current_date))
    }
    pub fn init(&self) -> Result<(), fern::InitError> {
        // Ensure logs directory exists
        fs::create_dir_all(&self.log_dir)
            .map_err(|e| fern::InitError::Io(io::Error::new(io::ErrorKind::Other, e)))?;

        let log_file = self.log_file_path();

        Dispatch::new()
            .level(LevelFilter::Debug)
            .chain(std::io::stdout())
            .chain(fern::log_file(log_file)?)
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{} [{}][{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .apply()?;

        Ok(())
    }
    pub fn update_daily_log(&mut self) -> Result<(), fern::InitError> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        if today != self.current_date {
            self.current_date = today;
            // Re-initialize logger to new file
            self.init()?;
        }
        Ok(())
    }

}