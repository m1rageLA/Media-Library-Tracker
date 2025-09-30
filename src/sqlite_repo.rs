use crate::models::{Category, MediaItem, Query, SortField, SortOrder, Status};
use crate::repo::{RepoResult, Repository, Stats};
use chrono::{Local, TimeZone};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row, ToSql};
use std::path::Path;
use std::sync::Mutex;

pub struct SqliteRepo {
    conn: Mutex<Connection>,
}

impl SqliteRepo {
    pub fn new(path: &Path) -> Self {
        let conn = Connection::open(path).expect("Failed to open DB");
        Self {
            conn: Mutex::new(conn),
        }
    }
}

impl Repository for SqliteRepo {
    fn init(&self) -> RepoResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                category INTEGER NOT NULL,
                status INTEGER NOT NULL,
                rating INTEGER,
                notes TEXT,
                cover_path TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_media_title ON media(title);
            CREATE INDEX IF NOT EXISTS idx_media_category ON media(category);
            CREATE INDEX IF NOT EXISTS idx_media_status ON media(status);
            "#,
        )?;
        Ok(())
    }

    fn add(&self, item: &mut MediaItem) -> RepoResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO media (title, category, status, rating, notes, cover_path, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                item.title,
                cat_to_i(item.category),
                status_to_i(item.status),
                item.rating.map(|r| r as i64),
                item.notes,
                item.cover_path,
                item.created_at.timestamp(),
                item.updated_at.timestamp(),
            ],
        )?;
        let id = conn.last_insert_rowid();
        item.id = Some(id);
        Ok(id)
    }

    fn update(&self, item: &MediaItem) -> RepoResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE media SET title=?1, category=?2, status=?3, rating=?4, notes=?5, cover_path=?6, updated_at=?7 WHERE id=?8",
            params![
                item.title,
                cat_to_i(item.category),
                status_to_i(item.status),
                item.rating.map(|r| r as i64),
                item.notes,
                item.cover_path,
                item.updated_at.timestamp(),
                item.id,
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: i64) -> RepoResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM media WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get(&self, id: i64) -> RepoResult<Option<MediaItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, category, status, rating, notes, cover_path, created_at, updated_at FROM media WHERE id=?1",
        )?;
        let item = stmt
            .query_row(params![id], |row| Ok(row_to_item(row)))
            .optional()?;
        Ok(item)
    }

    fn list(&self, q: &Query) -> RepoResult<Vec<MediaItem>> {
        let mut sql = String::from(
            "SELECT id, title, category, status, rating, notes, cover_path, created_at, updated_at FROM media",
        );
        let mut where_clauses: Vec<&str> = vec![];
        let mut params_dyn: Vec<Box<dyn ToSql>> = vec![];

        if !q.title_substr.trim().is_empty() {
            where_clauses.push("title LIKE ?");
            params_dyn.push(Box::new(format!("%{}%", q.title_substr.trim())));
        }
        if let Some(cat) = q.category {
            where_clauses.push("category = ?");
            params_dyn.push(Box::new(cat_to_i(cat)));
        }
        if let Some(st) = q.status {
            where_clauses.push("status = ?");
            params_dyn.push(Box::new(status_to_i(st)));
        }
        if let Some(minr) = q.min_rating {
            where_clauses.push("rating >= ?");
            params_dyn.push(Box::new(minr as i64));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }
        use SortField::*;
        use SortOrder::*;
        let order_by = match (q.sort_field, q.sort_order) {
            (Title, Asc) => "title ASC",
            (Title, Desc) => "title DESC",
            (Category, Asc) => "category ASC, title ASC",
            (Category, Desc) => "category DESC, title ASC",
            (Status, Asc) => "status ASC, updated_at DESC",
            (Status, Desc) => "status DESC, updated_at DESC",
            (Rating, Asc) => "rating ASC NULLS LAST, title ASC",
            (Rating, Desc) => "rating DESC NULLS LAST, title ASC",
            (CreatedAt, Asc) => "created_at ASC",
            (CreatedAt, Desc) => "created_at DESC",
            (UpdatedAt, Asc) => "updated_at ASC",
            (UpdatedAt, Desc) => "updated_at DESC",
        };
        sql.push_str(" ORDER BY ");
        sql.push_str(order_by);

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        let params_iter = params_from_iter(params_dyn.iter().map(|p| p.as_ref()));
        let rows = stmt.query_map(params_iter, |row| Ok(row_to_item(row)))?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn stats(&self) -> RepoResult<Stats> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM media", [], |r| r.get(0))?;

        let mut cat_stmt =
            conn.prepare("SELECT category, COUNT(*) FROM media GROUP BY category")?;
        let mut by_category = vec![];
        let cat_rows = cat_stmt.query_map([], |row| {
            let cat_i: i64 = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((i_to_cat(cat_i).as_str().to_string(), count as usize))
        })?;
        for row in cat_rows {
            by_category.push(row?);
        }

        let finished: i64 = conn.query_row(
            "SELECT COUNT(*) FROM media WHERE status = ?1",
            [status_to_i(Status::Finished)],
            |r| r.get(0),
        )?;
        let unfinished = total - finished;

        Ok(Stats {
            total: total as usize,
            by_category,
            finished: finished as usize,
            unfinished: unfinished as usize,
        })
    }
}

fn row_to_item(row: &Row<'_>) -> MediaItem {
    let id: i64 = row.get(0).unwrap();
    let title: String = row.get(1).unwrap();
    let category: i64 = row.get(2).unwrap();
    let status: i64 = row.get(3).unwrap();
    let rating: Option<i64> = row.get(4).unwrap();
    let notes: Option<String> = row.get(5).unwrap();
    let cover_path: Option<String> = row.get(6).unwrap();
    let created_at: i64 = row.get(7).unwrap();
    let updated_at: i64 = row.get(8).unwrap();

    MediaItem {
        id: Some(id),
        title,
        category: i_to_cat(category),
        status: i_to_status(status),
        rating: rating.map(|r| r as u8),
        notes,
        cover_path,
        created_at: Local.timestamp_opt(created_at, 0).unwrap(),
        updated_at: Local.timestamp_opt(updated_at, 0).unwrap(),
    }
}

fn cat_to_i(c: Category) -> i64 {
    match c {
        Category::Book => 0,
        Category::Movie => 1,
        Category::Game => 2,
        Category::Music => 3,
        Category::Other => 4,
    }
}

fn i_to_cat(i: i64) -> Category {
    match i {
        0 => Category::Book,
        1 => Category::Movie,
        2 => Category::Game,
        3 => Category::Music,
        _ => Category::Other,
    }
}

fn status_to_i(s: Status) -> i64 {
    match s {
        Status::Planned => 0,
        Status::InProgress => 1,
        Status::Finished => 2,
    }
}

fn i_to_status(i: i64) -> Status {
    match i {
        1 => Status::InProgress,
        2 => Status::Finished,
        _ => Status::Planned,
    }
}
