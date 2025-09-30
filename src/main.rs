mod app;
mod models;
mod repo;
mod sqlite_repo;
mod util;

use app::CatalogApp;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

fn main() -> eframe::Result<()> {
    // Basic logger (won't crash the app if it fails)
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    // Create/open local DB file next to the binary
    let db_path = util::default_db_path();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Media Catalog (Local)",
        options,
        Box::new(move |cc| Box::new(CatalogApp::new(cc, db_path.as_path()))),
    )
}
