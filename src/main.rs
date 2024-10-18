mod services;
mod utils;
mod entities;
mod enums;

use actix_web::{App, HttpServer, web::Data};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use services::{fetch_all_articles, fetch_article, create_article, update_article, delete_article};
use colored::*;
use crate::utils::log_with_colors;

pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    sqlx::migrate!("./migrations") // Path to your migrations directory
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log_with_colors("INFO", "Database migrations added successfully");

    println!(
        "{}", r#"
 /$$   /$$                     /$$                                       /$$
| $$  | $$                    | $$                                      | $$
| $$  | $$  /$$$$$$   /$$$$$$ | $$$$$$$   /$$$$$$   /$$$$$$   /$$$$$$$ /$$$$$$   /$$   /$$  /$$$$$$$
| $$$$$$$$ /$$__  $$ /$$__  $$| $$__  $$ |____  $$ /$$__  $$ /$$_____/|_  $$_/  | $$  | $$ /$$_____/
| $$__  $$| $$$$$$$$| $$  \ $$| $$  \ $$  /$$$$$$$| $$$$$$$$|  $$$$$$   | $$    | $$  | $$|  $$$$$$
| $$  | $$| $$_____/| $$  | $$| $$  | $$ /$$__  $$| $$_____/ \____  $$  | $$ /$$| $$  | $$ \____  $$
| $$  | $$|  $$$$$$$| $$$$$$$/| $$  | $$|  $$$$$$$|  $$$$$$$ /$$$$$$$/  |  $$$$/|  $$$$$$/ /$$$$$$$/
|__/  |__/ \_______/| $$____/ |__/  |__/ \_______/ \_______/|_______/    \___/   \______/ |_______/
                    | $$
                    | $$
                    |__/"#.cyan());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(fetch_all_articles)
            .service(fetch_article)
            .service(create_article)
            .service(update_article)
            .service(delete_article)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
