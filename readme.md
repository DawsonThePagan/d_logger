# d_logger

A rust crate providing simple logging with a dated clean up util.

## Examples

Create a logger with 7 days of cleaning

```rust
use d_logger::Logger;

let logger = Logger::new("C:/logs/".to_string(), "Log%d%m%y.log".to_string(), "%Y-%m-%d %H:%M:%S".to_string(), Some(7)).unwrap();
if !logger.write_log("This is a test log entry") {
    panic!("Logger failed");
}
// This will work
logger.log_clean(None);
```

Create a logger without any cleaning

```rust
use d_logger::Logger;

let logger = Logger::new("C:/logs/".to_string(), "Log%d%m%y.log".to_string(), "%Y-%m-%d %H:%M:%S".to_string(), None).unwrap();
if !logger.write_log("This is a test log entry") {
    panic!("Logger failed");
}
// This will fail
logger.log_clean(None);
```

Clean logs with a regex filter

```rust
use d_logger::Logger;

let logger = Logger::new("C:/logs/".to_string(), "Log%d%m%y.log".to_string(), "%Y-%m-%d %H:%M:%S".to_string(), Some(7)).unwrap();
if !logger.write_log("This is a test log entry") {
    panic!("Logger failed");
}
// This will work
logger.log_clean(Some(r"example_test_\d{8}.log"));
```

## Functions

### new(path: String, file_name_format: String, line_date_format: String, days_keep: Option<u64>) -> Result<Logger, io::error>

Create a new logging structure and test that the path and file given can be accessed.

### write_log(line: String) -> bool

Write a line to the log file. Will return whether its successful

### log_clean(regex: Option<&str>)

Clean up the log directory. If regex is provided only files matching regex will be deleted.
Will only delete anything if days_keep was set. The files delete must be older than the number of days wanted to keep.
