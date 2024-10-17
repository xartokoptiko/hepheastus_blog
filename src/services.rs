use actix_web::{
    get, post, put, delete,
    web::{Data, Json, Path},
    Responder, HttpResponse,
};
use serde::{ Serialize, Deserialize};
use sqlx::{self, FromRow};
use crate::AppState;

#[derive(Serialize, Deserialize, FromRow)]
struct Article{
    id : i8,
    title: String,
    description:String,
    md_filename:String,
    photo_filename:String,
}


#[get("/articles")]
pub async fn fetch_all_articles() -> impl Responder {
    "GET /articles".to_string()
}

#[get("/articles/{id}")]
pub async fn fetch_article(id: Path<i8>) -> impl Responder {
    "GET /articles/{id}".to_string()
}

#[post("/articles/{id}")]
pub async fn create_article() -> impl Responder {
    "POST /articles/".to_string()
}

#[put("/articles/{id}")]
pub async fn update_article(id: Path<i8>) -> impl Responder {
    "PUT /articles/{id}".to_string()
}

#[delete("/articles/{id}")]
pub async fn delete_article(id: Path<i8>) -> impl Responder {
    "DELETE /articles/{id}".to_string()
}