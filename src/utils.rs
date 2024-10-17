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