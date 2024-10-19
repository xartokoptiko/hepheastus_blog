use std::fmt::format;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use actix_multipart::Multipart;
use actix_web::{get, post, put, delete, web::{Data, Json, Path}, Responder, HttpResponse, Error};
use sqlx::{self, Row};
use crate::{entities, utils, AppState};
use utils::{log_with_colors, read_file_contents, read_photo_as_base64};
use entities::{ArticleEntity, ArticleCreateRequest, ArticleResponse};
use futures_util::stream::StreamExt;
use serde_json; // Ensure you have this for JSON handling
use sqlx::Error as SqlxError;
use std::fs::metadata;


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
        Err(e) => {
            log_with_colors("ERROR", &format!("Database query failed: {}", e));
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
    // Fetch the article from the database
    match sqlx::query_as::<_, ArticleEntity>(
        "SELECT * FROM articles WHERE id = $1"
    )
        .bind(id.into_inner())
        .fetch_one(&state.db)
        .await
    {
        Ok(article) => {
            // Construct file paths for markdown and photo
            let md_file_path = format!("/home/labros/hephaestus-blog/articles/{}/{}.md", article.id, article.id);
            let photo_file_path = format!("/home/labros/hephaestus-blog/articles/{}/{}.jpg", article.id, article.id);

            // Read the markdown file contents
            let md_contents = read_file_contents(&md_file_path).unwrap_or_else(|e| {
                log::error!("Failed to read markdown file: {}", e);
                log_with_colors("ERROR", &format!("Failed to read markdown file: {}", e));
                String::new() // Return an empty string or handle error as needed
            });

            // Read the photo file contents (as base64 string for JSON response)
            let photo_contents = read_photo_as_base64(&photo_file_path).unwrap_or_else(|e| {
                log::error!("Failed to read photo file: {}", e);
                log_with_colors("ERROR", &format!("Failed to read photo file: {}", e));
                String::new() // Return an empty string or handle error as needed
            });

            // Create a response struct to include article data and file contents
            let response = ArticleResponse {
                article,
                md_contents,
                photo_contents,
            };

            HttpResponse::Ok().json(response)
        },
        Err(_) => HttpResponse::NotFound().body("Article not found")
    }
}

#[post("/articles")]
pub async fn create_article(
    state: Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut new_article: Option<ArticleCreateRequest> = None;
    let mut markdown_content = None;
    let mut photo_data = None;

    // Loop through the multipart fields
    while let Some(field) = payload.next().await {
        let mut field = field?;
        // Extract content disposition outside the mutable borrow
        let content_disposition = field.content_disposition().unwrap();
        let field_name = content_disposition.get_name().unwrap().to_string(); // Store the field_name as a String

        // Handle the article JSON field
        if field_name == "article" {
            let mut json_string = String::new();
            while let Some(chunk) = field.next().await {
                json_string.push_str(&String::from_utf8_lossy(&chunk?));
            }
            // Deserialize the JSON string into your struct
            new_article = Some(serde_json::from_str(&json_string)?);
        }

        // Handle the markdown file
        if field_name == "markdown" {
            let mut content = String::new();
            while let Some(chunk) = field.next().await {
                content.push_str(&String::from_utf8_lossy(&chunk?));
            }
            markdown_content = Some(content);
        }

        // Handle the photo file
        if field_name == "photo" {
            let mut photo_bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                photo_bytes.extend_from_slice(&chunk?);
            }
            photo_data = Some(photo_bytes);
        }
    }

    // Ensure the new_article is populated before proceeding
    let new_article = new_article.ok_or_else(|| {
        log::error!("Missing article data");
        actix_web::error::ErrorBadRequest("Missing article data")
    })?;

    // Insert the article into the database
    let id = match sqlx::query(
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
        .await {
        Ok(record) => record.try_get::<i32, _>("id").unwrap(),
        Err(e) => {
            log::error!("Failed to create article: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to create article"));
        }
    };

    // Create the upload directory
    let upload_dir = format!("/home/labros/hephaestus-blog/articles/{}", id);
    create_dir_all(&upload_dir).map_err(|e| {
        log_with_colors("ERROR", &format!("Failed to create upload directory: {}", e));
        actix_web::error::ErrorInternalServerError("Failed to create upload directory")
    })?;

    // Handle markdown file creation
    if let Some(content) = markdown_content {
        let md_file_path = format!("{}/{}.md", upload_dir, id);
        let mut file = File::create(&md_file_path).map_err(|e| {
            log::error!("Failed to create markdown file: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create markdown file")
        })?;
        file.write_all(content.as_bytes()).map_err(|e| {
            log::error!("Failed to write to markdown file: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to write to markdown file")
        })?;
    }

    // Handle photo file creation
    if let Some(photo_bytes) = photo_data {
        let photo_file_path = format!("{}/{}.jpg", upload_dir, id);
        let mut file = File::create(&photo_file_path).map_err(|e| {
            log::error!("Failed to create photo file: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create photo file")
        })?;
        file.write_all(&photo_bytes).map_err(|e| {
            log::error!("Failed to write to photo file: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to write to photo file")
        })?;
    }

    // Create the ArticleEntity instance with the generated ID
    let article = ArticleEntity {
        id,
        title: new_article.title.clone(),
        description: new_article.description.clone(),
        md_filename: format!("{}.md", id), // Use the ID for filename
        photo_filename: format!("{}.jpg", id), // Use the ID for filename
        article_type: new_article.article_type,
    };

    // Update the article with the markdown and photo filenames
    match sqlx::query(
        r#"
        UPDATE articles
        SET md_filename = $1, photo_filename = $2
        WHERE id = $3
        "#
    )
        .bind(&article.md_filename)
        .bind(&article.photo_filename)
        .bind(id)
        .execute(&state.db)
        .await {
        Ok(_) => {
            log_with_colors("INFO", "POST 200 /articles");
            Ok(HttpResponse::Created().json(article))
        },
        Err(e) => {
            log::error!("Failed to update article filenames: {}", e);
            log_with_colors("WARN", "POST 404 /articles - Article title and description added, failed to add md and photo filename");
            Ok(HttpResponse::InternalServerError().body("Failed to update article filenames"))
        },
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