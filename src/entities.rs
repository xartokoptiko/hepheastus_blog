use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::enums::ArticleType;

#[derive(Serialize, Deserialize, FromRow)]
pub struct ArticleEntity {
    pub(crate) id : i32,
    pub(crate) title: String,
    pub(crate) description:String,
    pub(crate) md_filename:String,
    pub(crate) photo_filename:String,
    pub(crate) article_type:i32
}


pub struct Article {
    id: i32,
    title: String,
    description: String,
    md_filename: String,
    photo_filename: String,
    article_type: ArticleType
}

impl From<ArticleEntity> for Article {
    fn from(entity: ArticleEntity) -> Self {
        Article {
            id: entity.id,
            title: entity.title,
            description: entity.description,
            md_filename: entity.md_filename,
            photo_filename: entity.photo_filename,
            article_type: ArticleType::from(entity.article_type),
        }
    }
}

impl ArticleEntity {
    // Create an ArticleEntity from a successful insert and generated filenames
    pub fn from_insert(id: i32, title: String, description: String, article_type: i32) -> Self {
        let md_filename = format!("{}.md", id);
        let photo_filename = format!("{}.jpg", id); // Change extension if needed

        ArticleEntity {
            id,
            title,
            description,
            md_filename,
            photo_filename,
            article_type,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ArticleCreateRequest{
    pub(crate) title:String,
    pub(crate) description:String,
    pub(crate) article_type:i32
}