#![allow(dead_code)]
use std::env::consts::OS;
use std::io;
use std::{fs::{self, OpenOptions}, io::Write, path::Path, time::{SystemTime, UNIX_EPOCH}};
use chrono::prelude::*;
use regex::Regex;

const SECS_1_DAY: u64 = 86400;

const NEW_LINE_WINDOWS: &str = "\r\n";
const NEW_LINE_LINUX: &str = "\n";
const PATH_SPLIT_WINDOWS: &str = r"\";
const PATH_SPLIT_LINUX: &str = "/";

// Shorthand string maker and &str converter
macro_rules! toString {
    ($s:expr) => { $s.to_string() }
}
macro_rules! toAmpStr {
    ($s:expr) => { toString!($s).as_str() }
}

/// Logger will write log entries to a file with date and time in the line, using dated logs.
/// It can also clean up old log files if needed.
/// # Example
/// ```rust
/// use d_logger::Logger;
/// 
/// let logger = Logger::new("C:/logs/".to_string(), "Log%d%m%y.log".to_string(), "%Y-%m-%d %H:%M:%S".to_string(), Some(7)).unwrap();
/// if !logger.write_log("This is a test log entry") {
///    panic!("Logger failed");
/// }
/// logger.log_clean(None);
/// ```
pub struct Logger {
    /// Path to log file, must end with a \ on windows or / on linux
    path: String,
    /// Date format for file name, must contain full file name. e.g. Log%d%m%y.log
    file_name_format: String,
    /// Date format for lines
    line_date_format: String,
    /// Number of days to keep if using log clean
    days_keep: Option<u64>,
}

impl Logger {
    // Create a new logger and make sure we can use the log file given
    /// # Arguments
    /// * `path` - Path to log file, must end with a \ on windows or / on linux
    /// * `file_name_format` - Date format for file name, must contain full file name. e.g. Log%d%m%y.log
    /// * `line_date_format` - Date format for lines
    /// * `days_keep` - Number of days to keep if using log clean, set to None to disable
    /// # Example
    /// ```rust
    /// use d_logger::Logger;
    /// 
    /// let logger = Logger::new("C:/logs/".to_string(), "Log%d%m%y.log".to_string(), "%Y-%m-%d %H:%M:%S".to_string(), Some(7)).unwrap();
    /// if !logger.write_log("This is a test log entry") {
    ///     panic!("Logger failed");
    /// }
    /// logger.log_clean(None);
    /// ```
    pub fn new(path: String, file_name_format: String, line_date_format: String, days_keep: Option<u64>) -> Result<Logger, io::Error> {
        let new_line = match OS {
            "linux" => NEW_LINE_LINUX,
            "windows" => NEW_LINE_WINDOWS,
            _ => return Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS")),
        };

        if !Path::new(path.as_str()).exists() { // Check if the dir exists
            _ = match fs::create_dir(path.as_str()) { // Try to create it if it doesn't
                Ok(x) => x, 
                Err(e) => {return Err(e)}
            };
        }

        let now: DateTime<Local> = Local::now();
        let log_file_name = path.clone() + toAmpStr!(now.format(file_name_format.as_str())); // Get the log file date

        let mut file = match OpenOptions::new().append(true).create(true).open(log_file_name.as_str()) {
            Ok(x) => x, // Open the log file
            Err(e) => {return Err(e)}
        };
        _ = match file.write_all(new_line.as_bytes()) { // Write the new line
            Ok(x) => x,
            Err(e) => {return Err(e)}
        };

        Ok(Logger {path, file_name_format, line_date_format, days_keep})
    }

    /// Write a line to the log.
    /// # Arguments
    /// * `line` - The line to write to the log
    /// # Example
    /// ```rust
    /// use d_logger::Logger;
    /// 
    /// if !logger.write_log("This is a test log entry") {
    ///    panic!("Logger failed");
    /// }
    /// ```
    /// # Returns
    /// * `true` if the log was written successfully
    /// * `false` if the log could not be written
    pub fn write_log(&self, line: &str) -> bool {
        let new_line = match OS {
            "linux" => NEW_LINE_LINUX,
            "windows" => NEW_LINE_WINDOWS,
            _ => return false,
        };

        // Get time and format it 
        let now: DateTime<Local> = Local::now();
        let time = now.format(self.line_date_format.as_str()).to_string();
        let log_file_name = self.path.clone() + &now.format(self.file_name_format.as_str()).to_string();
        
        // Open log file
        let mut file = match OpenOptions::new().append(true).create(true).open(&log_file_name) {
            Ok(x) => x,
            Err(_) => return false,
        };

        // Print to console if we are debugging
        if cfg!(debug_assertions) {
            println!("{line}");
        }

        // Write everything to file
        let log_entry = format!("{}{}{}", time, line, new_line);
        if file.write_all(log_entry.as_bytes()).is_ok() && file.flush().is_ok() {
            file.sync_all().unwrap();
            true
        } else {
            false
        }
    }

    /// Clean up log path. Will not delete any log files if the days_keep is set to None.
    /// Will only delete files older than the days to keep.
    /// Provide Some(<regex>) to filter by name or None to delete any file older than date
    /// # Arguments
    /// * `filter` - Optional regex to filter by name, None to delete any file older than date
    /// # Example
    /// ```rust
    /// use d_logger::Logger;
    /// 
    /// logger.clean_log(None);
    /// 
    /// logger.clean_log(Some(r"example_test_\d{8}.log"));
    /// ```
    pub fn log_clean(&self, filter: Option<&str>) {
        let path_split = match OS {
            "linux" => PATH_SPLIT_LINUX,
            "windows" => PATH_SPLIT_WINDOWS,
            _ => return
        };

        let paths = match fs::read_dir(&self.path) {
            Ok(paths) => paths,
            Err(e) => {
                self.write_log(&format!("Error = Log cleaner, could not read directory: {e}"));
                return;
            }
        };

        if self.days_keep.is_none() {
            return;
        }

        let file_filter = filter.unwrap_or("");
        let now = SystemTime::now();
        let current_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let threshold = current_time - (self.days_keep.unwrap() * 86400);

        let regex = if !file_filter.is_empty() {
            Some(Regex::new(file_filter).unwrap())
        } else {
            None
        };

        for entry in paths.filter_map(Result::ok) {
            let file_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => {
                    self.write_log("Error = Log cleaner, could not convert file name");
                    continue;
                }
            };

            if let Some(ref regex) = regex {
                if !regex.is_match(&file_name) {
                    continue;
                }
            }

            let file_path = self.path.clone() + path_split + &file_name;

            if Path::new(&file_path).is_dir() {
                continue;
            }

            let metadata = match fs::metadata(&file_path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    self.write_log(&format!("Error = Log cleaner, could not read metadata from file {file_name} | {e}"));
                    continue;
                }
            };

            let modified_time = match metadata.modified() {
                Ok(modified) => match modified.duration_since(UNIX_EPOCH) {
                    Ok(duration) => duration.as_secs(),
                    Err(e) => {
                        self.write_log(&format!("Error = Log cleaner, could not get modified time for file {file_name} | {e}"));
                        continue;
                    }
                },
                Err(e) => {
                    self.write_log(&format!("Error = Log cleaner, could not read modified time from file {file_name} | {e}"));
                    continue;
                }
            };
            if modified_time < threshold {
                if let Err(e) = fs::remove_file(&file_path) {
                    self.write_log(&format!("Error = Log cleaner, could not delete file {file_name} | {e}"));
                }
            }
        }
    }
}