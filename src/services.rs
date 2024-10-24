use std::fs::{create_dir_all, File};
use std::io::Write;
use actix_multipart::Multipart;
use actix_web::{get, post, put, delete, web::{Data, Json, Path}, Responder, HttpResponse, Error};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::{self, Row};
use crate::{entities, utils, AppState};
use utils::{log_with_colors, read_file_contents, read_photo_as_base64};
use entities::{ArticleEntity, ArticleCreateRequest, ArticleResponse, LoginRequest, User};
use futures_util::stream::StreamExt;
use serde_json;
use crate::entities::SignupRequest;
use crate::utils::generate_jwt;

//#[get("/articles")]
pub async fn fetch_all_articles(state: Data<AppState>) -> impl Responder {

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
        }
        Err(e) => {
            log_with_colors("ERROR", &format!("Database query failed: {}", e));
            log_with_colors("WARN", "GET 404 /articles");
            HttpResponse::NotFound().body("No Articles found")
        }
    }
}

//#[get("/articles/{id}")]
pub async fn fetch_article(
    state: Data<AppState>,
    id: Path<i32>,
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
            // Construct file paths for markdown and photo// Get the home directory dynamically
            let home_dir = home::home_dir().expect("Failed to get home directory");
            let md_file_path = home_dir.join(format!("hephaestus-blog/articles/{}/{}.md", article.id, article.id));
            let photo_file_path = home_dir.join(format!("hephaestus-blog/articles/{}/{}.jpg", article.id, article.id));

            // Convert PathBuf to &str
            let md_file_path_str = md_file_path.to_str().expect("Failed to convert markdown file path to string");
            let photo_file_path_str = photo_file_path.to_str().expect("Failed to convert photo file path to string");

            // Read the markdown file contents
            let md_contents = read_file_contents(&md_file_path_str).unwrap_or_else(|e| {
                log_with_colors("ERROR", &format!("Failed to read markdown file: {}", e));
                String::new() // Return an empty string or handle error as needed
            });

            // Read the photo file contents (as base64 string for JSON response)
            let photo_contents = read_photo_as_base64(&photo_file_path_str).unwrap_or_else(|e| {
                log_with_colors("ERROR", &format!("Failed to read photo file: {}", e));
                String::new() // Return an empty string or handle error as needed
            });

            // Create a response struct to include article data and file contents
            let response = ArticleResponse {
                article,
                md_contents,
                photo_contents,
            };

            log_with_colors("INFO", "GET 200 articles/{id}");
            HttpResponse::Ok().json(response)
        }
        Err(_) => {
            log_with_colors("WARN", "GET 404 /articles/{id}");
            HttpResponse::NotFound().body("Article not found")
        }
    }
}

//#[post("/articles")]
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
        log_with_colors("WARN", "POST 404 articles - Missing article data");
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
            log_with_colors("ERROR", &format!("Failed to create article: {}", e));
            return Ok(HttpResponse::InternalServerError().body("Failed to create article"));
        }
    };

    // Create the upload directory
    let home_dir = home::home_dir().expect("Failed to get home directory");
    let upload_dir_buf = home_dir.join("upload");
    let upload_dir = upload_dir_buf.to_str().expect("Failed to convert upload dir to string");
    create_dir_all(&upload_dir).map_err(|e| {
        log_with_colors("ERROR", &format!("Failed to create upload directory: {}", e));
        actix_web::error::ErrorInternalServerError("Failed to create upload directory")
    })?;

    // Handle markdown file creation
    if let Some(content) = markdown_content {
        let md_file_path = format!("{}/{}.md", upload_dir, id);
        let mut file = File::create(&md_file_path).map_err(|e| {
            log_with_colors("ERROR", &format!("Failed to create markdown file: {}", e));
            actix_web::error::ErrorInternalServerError("Failed to create markdown file")
        })?;
        file.write_all(content.as_bytes()).map_err(|e| {
            log_with_colors("ERROR", &format!("Failed to write markdown file: {}", e));
            actix_web::error::ErrorInternalServerError("Failed to write to markdown file")
        })?;
    }

    // Handle photo file creation
    if let Some(photo_bytes) = photo_data {
        let photo_file_path = format!("{}/{}.jpg", upload_dir, id);
        let mut file = File::create(&photo_file_path).map_err(|e| {
            log_with_colors("ERROR", &format!("Failed to create photo file: {}", e));
            actix_web::error::ErrorInternalServerError("Failed to create photo file")
        })?;
        file.write_all(&photo_bytes).map_err(|e| {
            log_with_colors("ERROR", &format!("Failed to write photo file: {}", e));
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
        }
        Err(e) => {
            log_with_colors("WARN", "POST 404 /articles - Article title and description added, failed to add md and photo filename");
            Ok(HttpResponse::InternalServerError().body("Failed to update article filenames"))
        }
    }
}


//TODO NEEDS TESTING
//#[put("/articles/{id}")]
pub async fn update_article(
    state: Data<AppState>,
    id: Path<i32>,
    updated_article: Json<ArticleEntity>,
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
        }
        Ok(_) => {
            log_with_colors("WARN", "PUT 404 /article");
            HttpResponse::NotFound().body("Article not found")
        }
        Err(_) => {
            log_with_colors("ERROR", "PUT 500 /article");
            HttpResponse::InternalServerError().body("Failed to update article")
        }
    }
}


//TODO NEED TO DELETE THE FILES AS WELL AND NEEDS TESTING
//#[delete("/articles/{id}")]
pub async fn delete_article(
    state: Data<AppState>,
    id: Path<i32>,
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
        }
        Ok(_) => {
            log_with_colors("WARN", "DELETE 404 /article");
            HttpResponse::NotFound().body("Article not found")
        }
        Err(_) => {
            log_with_colors("ERROR", "DELETE 500 /article");
            HttpResponse::InternalServerError().body("Failed to delete article")
        }
    }
}


// LOGIN SERVICES
pub async fn login(db_pool: Data<AppState>, data: Json<LoginRequest>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&data.email)
        .fetch_optional(&db_pool.db)  // Use &db_pool.db to get the actual connection pool
        .await
        .unwrap();  // Handle errors appropriately in production

    if let Some(user) = user {
        let password_matches = verify(&data.password, &user.password_hash).unwrap(); // Verifies hashed password

        if password_matches {
            let token = generate_jwt(&user.email).unwrap();  // Generate a JWT token
            return HttpResponse::Ok().json(serde_json::json!({ "token": token }));  // Return the token as a JSON response
        }
    }

    // If authentication fails, return unauthorized
    HttpResponse::Unauthorized().finish()
}

pub async fn signup(db_pool: Data<AppState>, data: Json<SignupRequest>) -> impl Responder {
    // Check if the user already exists
    let user_exists = sqlx::query("SELECT 1 FROM users WHERE email = $1")
        .bind(&data.email)
        .fetch_optional(&db_pool.db)
        .await
        .unwrap()
        .is_some();

    if user_exists {
        return HttpResponse::BadRequest().body("User already exists");
    }

    // Hash the password
    let hashed_password = hash(&data.password, DEFAULT_COST).unwrap();

    // Insert the new user into the database
    sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
        .bind(&data.email)
        .bind(&hashed_password)
        .execute(&db_pool.db)
        .await
        .unwrap();

    HttpResponse::Ok().body("User created")
}
