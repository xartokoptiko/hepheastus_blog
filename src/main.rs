mod services;

use actix_web::{App, HttpServer, web::Data};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

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


    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
