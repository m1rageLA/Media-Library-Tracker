use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Book,
    Movie,
    Game,
    Music,
    Other,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Book,
        Category::Movie,
        Category::Game,
        Category::Music,
        Category::Other,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Book => "Book",
            Category::Movie => "Movie",
            Category::Game => "Game",
            Category::Music => "Music",
            Category::Other => "Other",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Planned,
    InProgress,
    Finished,
}

impl Status {
    pub const ALL: [Status; 3] = [Status::Planned, Status::InProgress, Status::Finished];

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Planned => "Planned",
            Status::InProgress => "In Progress",
            Status::Finished => "Finished",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Option<i64>,
    pub title: String,
    pub category: Category,
    pub status: Status,
    pub rating: Option<u8>,
    pub notes: Option<String>,
    pub cover_path: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl MediaItem {
    pub fn new(title: impl Into<String>, category: Category) -> Self {
        let now = Local::now();
        Self {
            id: None,
            title: title.into(),
            category,
            status: Status::Planned,
            rating: None,
            notes: None,
            cover_path: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_finished(&mut self) {
        self.status = Status::Finished;
        self.updated_at = Local::now();
    }

    pub fn set_rating(&mut self, rating: Option<u8>) {
        self.rating = rating;
        self.updated_at = Local::now();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Title,
    Category,
    Status,
    Rating,
    CreatedAt,
    UpdatedAt,
}

impl Default for SortField {
    fn default() -> Self {
        SortField::Title
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Asc
    }
}

#[derive(Debug, Clone)]
pub struct Query {
    pub title_substr: String,
    pub category: Option<Category>,
    pub status: Option<Status>,
    pub min_rating: Option<u8>,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            title_substr: String::new(),
            category: None,
            status: None,
            min_rating: None,
            sort_field: SortField::default(),
            sort_order: SortOrder::default(),
        }
    }
}
