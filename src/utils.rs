use std::fs::File;
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use colored::*;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, errors::Error as JwtError};
use std::env;
use bcrypt::{hash, DEFAULT_COST};
use sqlx::PgPool;
use crate::entities;
use entities::Claims;

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

// FILE UTILS

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


// JWT UTILS

const JWT_EXPIRATION: usize = 60 * 60; // Token expiration time in seconds (1 hour)

pub fn generate_jwt(user_email: &str) -> Result<String, JwtError> {
    let expiration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + JWT_EXPIRATION;
    let claims = Claims {
        sub: user_email.to_owned(),
        exp: expiration,
    };

    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
}

pub fn validate_jwt(token: &str) -> Result<Claims, JwtError> {
    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
        .map(|data| data.claims)
}

// GENERAL UTILS
pub async fn create_default_user_if_not_exists(db_pool: &PgPool) {
    // Load email and password from environment variables
    let default_email = env::var("DEFAULT_USER_EMAIL").expect("DEFAULT_USER_EMAIL must be set");
    let default_password = env::var("DEFAULT_USER_PASSWORD").expect("DEFAULT_USER_PASSWORD must be set");

    // Check if the default user exists
    let user_exists = sqlx::query("SELECT 1 FROM users WHERE email = $1")
        .bind(&default_email)
        .fetch_optional(db_pool)
        .await
        .unwrap()
        .is_some();

    if !user_exists {
        // Hash the default password
        let hashed_password = hash(&default_password, DEFAULT_COST).unwrap();

        // Insert the default user into the database
        sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
            .bind(&default_email)
            .bind(&hashed_password)
            .execute(db_pool)
            .await
            .unwrap();

        log_with_colors("INFO", format!("Created user {}", default_email).as_str())
    } else {
        log_with_colors("INFO", "Default user already exists.");
    }
}