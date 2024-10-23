mod services;
mod utils;
mod entities;
mod enums;
mod auth;

use actix_web::{App, HttpServer, web::Data};
use actix_web::web::{delete, get, post, put, route, scope};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use services::{fetch_all_articles, fetch_article, create_article, update_article, delete_article};
use colored::*;
use crate::services::{login, signup};
use crate::utils::{create_default_user_if_not_exists, log_with_colors};

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

    create_default_user_if_not_exists(&pool).await;

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
            .route("/auth/sign-in", post().to(login))
            .route("/articles", get().to(fetch_all_articles))
            .route("/articles/{article_id}", get().to(fetch_article))
            .service(
                scope("/protected")
                    .wrap(auth::Auth)
                    .route("/articles", post().to(create_article))
                    .route("/articles/{id}", put().to(update_article))
                    .route("/articles/{id}", delete().to(delete_article))
                    .route("/sign-up", post().to(signup))
            )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
