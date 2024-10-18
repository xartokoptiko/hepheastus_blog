use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ArticleType{
    Important,
    Favourite,
    Common,
}

impl From<i32> for ArticleType {
    fn from(value: i32) -> Self {
        match value {
            0 => ArticleType::Important,
            1 => ArticleType::Favourite,
            2 => ArticleType::Common,
            _ => panic!("Invalid ArticleType value"),
        }
    }
}

impl Into<i32> for ArticleType {
    fn into(self) -> i32 {
        match self {
            ArticleType::Important => 0,
            ArticleType::Favourite => 1,
            ArticleType::Common => 2,
        }
    }
}