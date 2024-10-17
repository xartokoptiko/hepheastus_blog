use actix_web::{
    get, post, put, delete,
    web::{Data, Json, Path},
    Responder, HttpResponse,
};
use serde::{ Serialize, Deserialize};
use sqlx::{self, FromRow};
use crate::{utils, AppState};
use utils::log_with_colors;

#[derive(Serialize, Deserialize, FromRow)]
struct Article{
    id : i32,
    title: String,
    description:String,
    md_filename:String,
    photo_filename:String,
}


#[get("/articles")]
pub async fn fetch_all_articles(state : Data<AppState>) -> impl Responder {

    //"GET /articles".to_string()

    match sqlx::query_as::<_, Article>(
        "SELECT * FROM articles"
    )
        .fetch_all(&state.db)
        .await
    {
        Ok(articles) => {
            log_with_colors("INFO", "GET 200 /articles");
            HttpResponse::Ok().json(articles)
        },
        Err(_) => {
            log_with_colors("WARN", "GET 404 /articles");
            HttpResponse::NotFound().body("No Articles found")
        }
    }

}

#[get("/articles/{id}")]
pub async fn fetch_article(
    state: Data<AppState>,
    id: Path<i32>
) -> impl Responder {

    match sqlx::query_as::<_, Article>(
        "SELECT * FROM articles WHERE id = $1"
    )
        .bind(id.into_inner())  // Bind the path parameter to the query
        .fetch_one(&state.db)
        .await
    {
        Ok(article) => HttpResponse::Ok().json(article),
        Err(_) => HttpResponse::NotFound().body("Article not found")
    }
}


//Todo This function here is for testing purposes only. This means that it needs to be changed
// for the desired outcome

#[post("/articles")]
pub async fn create_article(
    state: Data<AppState>,
    new_article: Json<Article>
) -> impl Responder {

    let article = new_article.into_inner();

    match sqlx::query(
        "INSERT INTO articles (title, description, md_filename, photo_filename) VALUES ($1, $2, $3, $4)"
    )
        .bind(&article.title)
        .bind(&article.description)
        .bind(&article.md_filename)
        .bind(&article.photo_filename)
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Created().body("Article created successfully"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create article"),
    }
}


//Todo This function here is for testing purposes only. This means that it needs to be changed for
// the desired outcome

#[put("/articles/{id}")]
pub async fn update_article(
    state: Data<AppState>,
    id: Path<i32>,
    updated_article: Json<Article>
) -> impl Responder {

    let article = updated_article.into_inner();

    match sqlx::query(
        "UPDATE articles SET title = $1, description = $2, md_filename = $3, photo_filename = $4 WHERE id = $5"
    )
        .bind(&article.title)
        .bind(&article.description)
        .bind(&article.md_filename)
        .bind(&article.photo_filename)
        .bind(id.into_inner())  // Bind the path parameter to the query
        .execute(&state.db)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().body("Article updated successfully"),
        Ok(_) => HttpResponse::NotFound().body("Article not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update article"),
    }
}


//This function here is for testing purposes only. This means that it needs to be changed for
//the desired outcome

#[delete("/articles/{id}")]
pub async fn delete_article(
    state: Data<AppState>,
    id: Path<i32>
) -> impl Responder {

    match sqlx::query(
        "DELETE FROM articles WHERE id = $1"
    )
        .bind(id.into_inner())  // Bind the id parameter to the query
        .execute(&state.db)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().body("Article deleted successfully"),
        Ok(_) => HttpResponse::NotFound().body("Article not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete article"),
    }
}
