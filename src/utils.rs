use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::SystemTime;
use colored::*;

// Function to simulate Spring Boot-style logging with timestamp and colors
pub fn log_with_colors(level: &str, message: &str) {
    let now = SystemTime::now();
    let datetime: chrono::DateTime<chrono::Local> = now.into();

    let log_level = match level {
        "INFO" => level.green(),
        "WARN" => level.yellow(),
        "ERROR" => level.red(),
        _ => level.white(),
    };

    println!(
        "{} [{}] - {}",
        datetime.format("%Y-%m-%d %H:%M:%S"),
        log_level,
        message
    );
}

pub fn read_file_contents(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Function to read photo as base64 string
pub fn read_photo_as_base64(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Convert the bytes to base64 and return as Result
    Ok(base64::encode(&buffer))
}
