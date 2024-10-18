use actix_web::{
    get, post, put, delete,
    web::{Data, Json, Path},
    Responder, HttpResponse,
};
use sqlx::{self, Row};
use crate::{entities, utils, AppState};
use utils::log_with_colors;
use entities::{ArticleEntity, ArticleCreateRequest};

#[get("/articles")]
pub async fn fetch_all_articles(state : Data<AppState>) -> impl Responder {

    //"GET /articles".to_string()

    match sqlx::query_as::<_, ArticleEntity>(
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

    match sqlx::query_as::<_, ArticleEntity>(
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

#[post("/articles")]
pub async fn create_article(
    state: Data<AppState>,
    new_article: Json<ArticleCreateRequest>
) -> impl Responder {
    // Insert the article into the database
    match sqlx::query(
        r#"
        INSERT INTO articles (title, description, article_type)
        VALUES ($1, $2, $3)
        RETURNING id
        "#
    )
        .bind(&new_article.title)
        .bind(&new_article.description)
        .bind(new_article.article_type)
        .fetch_one(&state.db)
        .await
    {
        Ok(record) => {
            let id: i32 = record.try_get("id").unwrap();

            // Create the ArticleEntity instance with the generated ID
            let article = ArticleEntity {
                id,
                title: new_article.title.clone(),
                description: new_article.description.clone(),
                md_filename: format!("{}.md", id), // Generated filename
                photo_filename: format!("{}.jpg", id), // Generated filename
                article_type: new_article.article_type,
            };

            // Update the article if needed (this may be optional)
            match sqlx::query(
                r#"
                UPDATE articles
                SET md_filename = $1, photo_filename = $2
                WHERE id = $3
                "#
            )
                .bind(&article.md_filename) // Use generated filename
                .bind(&article.photo_filename) // Use generated filename
                .bind(id)
                .execute(&state.db)
                .await
            {
                Ok(_) => {
                    log_with_colors("INFO", "POST 200 /articles");
                    HttpResponse::Created().json(article)
                }, // Respond with the created article
                Err(_) => {
                    log_with_colors("WARN", "POST 404 /articles - Article title and description added, failed to add md and photo filename");
                    HttpResponse::InternalServerError().body("Failed to update article filenames")
                }, // Handle update error
            }
        },
        Err(_) => HttpResponse::InternalServerError().body("Failed to create article"), // Handle insert error
    }
}

#[put("/articles/{id}")]
pub async fn update_article(
    state: Data<AppState>,
    id: Path<i32>,
    updated_article: Json<ArticleEntity>
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
        Ok(result) if result.rows_affected() > 0 => {
            log_with_colors("INFO", "PUT 200 /article");
            HttpResponse::Ok().body("Article updated successfully")
        },
        Ok(_) => {
            log_with_colors("WARN", "PUT 404 /article");
            HttpResponse::NotFound().body("Article not found")
        },
        Err(_) => {
            log_with_colors("ERROR", "PUT 500 /article");
            HttpResponse::InternalServerError().body("Failed to update article")
        },
    }
}

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
        Ok(result) if result.rows_affected() > 0 => {
            log_with_colors("INFO", "DELETE 200 /article");
            HttpResponse::Ok().body("Article deleted successfully")
        },
        Ok(_) => {
            log_with_colors("WARN", "DELETE 404 /article");
            HttpResponse::NotFound().body("Article not found")
        },
        Err(_) => {
            log_with_colors("ERROR", "DELETE 500 /article");
            HttpResponse::InternalServerError().body("Failed to delete article")
        },
    }
}