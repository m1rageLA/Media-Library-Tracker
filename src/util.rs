use crate::models::MediaItem;
use chrono::Local;
use std::fs::File;
use std::path::PathBuf;

pub fn default_db_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("media_catalog.sqlite");
    path
}

pub fn export_csv(items: &[MediaItem]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut out = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let filename = format!("export_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
    out.push(filename);

    let file = File::create(&out)?;
    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record([
        "id",
        "title",
        "category",
        "status",
        "rating",
        "notes",
        "cover_path",
        "created_at",
        "updated_at",
    ])?;
    for item in items {
        wtr.write_record([
            item.id.map(|v| v.to_string()).unwrap_or_default(),
            item.title.clone(),
            item.category.to_string(),
            item.status.to_string(),
            item.rating.map(|v| v.to_string()).unwrap_or_default(),
            item.notes.clone().unwrap_or_default(),
            item.cover_path.clone().unwrap_or_default(),
            item.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            item.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ])?;
    }
    wtr.flush()?;

    Ok(out)
}
