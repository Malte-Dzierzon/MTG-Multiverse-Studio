//! MTG Multiverse Studio — Bulk Import CLI
//!
//! Command-line tool to import Scryfall card data and lore entries into the local SQLite database.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin mtg-import -- oracle
//! cargo run --bin mtg-import -- default-cards
//! cargo run --bin mtg-import -- oracle --db /custom/path/mtg.db
//! cargo run --bin mtg-import -- lore [--path assets/stories/]
//! ```

use clap::Parser;
use std::path::PathBuf;

#[path = "../import_engine/mod.rs"]
mod import_engine;
#[path = "../db/mod.rs"]
mod db;
#[path = "../services/mod.rs"]
mod services;
#[path = "../utils/mod.rs"]
mod utils;
#[path = "../models.rs"]
mod models;
#[path = "../scryfall/mod.rs"]
mod scryfall;

#[derive(clap::ValueEnum, Debug, Clone)]
enum DatasetArg {
    Oracle,
    #[clap(name = "default-cards")]
    DefaultCards,
    Rulings,
    Sets,
    Lore,
}

impl From<DatasetArg> for import_engine::Dataset {
    fn from(arg: DatasetArg) -> Self {
        match arg {
            DatasetArg::Oracle => import_engine::Dataset::OracleCards,
            DatasetArg::DefaultCards => import_engine::Dataset::DefaultCards,
            DatasetArg::Rulings => import_engine::Dataset::Rulings,
            DatasetArg::Sets => import_engine::Dataset::Sets,
            DatasetArg::Lore => {
                // Not a real Dataset variant; Lore is handled separately
                import_engine::Dataset::OracleCards
            }
        }
    }
}

/// MTG Multiverse Studio — Scryfall Bulk Import & Lore Import Tool
#[derive(Parser, Debug)]
#[command(name = "mtg-import", version, about)]
struct Cli {
    /// Dataset to import: oracle | default-cards | rulings | sets | lore
    #[arg(value_enum)]
    dataset: DatasetArg,

    /// Path to SQLite database (default: mtg_multiverse_studio.db)
    #[arg(short, long)]
    db: Option<PathBuf>,

    /// Path to lore stories directory (only used with 'lore' dataset)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn run_lore_import(conn: &rusqlite::Connection, stories_dir: &PathBuf, verbose: bool) {
    println!("📖 Starte Lore-Import aus: {}", stories_dir.display());

    if !stories_dir.exists() {
        eprintln!("⚠️  Verzeichnis existiert nicht: {}", stories_dir.display());
        return;
    }

    let mut imported = 0u64;
    let mut skipped = 0u64;
    let mut failed = 0u64;

    // Scan the top-level directory for .md files
    let read_dir = match std::fs::read_dir(stories_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("❌ Fehler beim Lesen des Verzeichnisses: {}", e);
            return;
        }
    };

    // Also collect from language subdirectories (en/, de/)
    let subdirs: Vec<PathBuf> = ["en", "de"]
        .iter()
        .map(|lang| stories_dir.join(lang))
        .filter(|p| p.exists())
        .collect();

    let mut files: Vec<PathBuf> = Vec::new();

    // Files from top-level
    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            files.push(path);
        }
    }

    // Files from subdirectories
    for subdir in &subdirs {
        if let Ok(read_dir) = std::fs::read_dir(subdir) {
            for entry in read_dir {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    files.push(path);
                }
            }
        }
    }

    // Skip README.md
    files.retain(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n != "README.md")
            .unwrap_or(true)
    });

    println!("📄 Gefundene Story-Dateien: {}", files.len());

    // Collect existing content_paths to deduplicate
    let existing_paths: std::collections::HashSet<String> = db::lore_repo::get_all_lore_entries(conn, None)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.content_path)
        .collect();

    for file_path in &files {
        let path_str = file_path.to_string_lossy().to_string();

        // Skip already imported
        if existing_paths.contains(&path_str) {
            skipped += 1;
            if verbose {
                println!("  ⏭️  Übersprungen (bereits importiert): {}", file_path.display());
            }
            continue;
        }

        match services::lore_service::parse_lore_file(file_path) {
            Ok(Some(entry)) => {
                let metadata_str = serde_json::to_string(&entry.metadata).unwrap_or_else(|_| "{}".to_string());
                let related_str = serde_json::to_string(&entry.related_cards).unwrap_or_else(|_| "[]".to_string());

                match db::lore_repo::insert_lore_entry(
                    conn,
                    &entry.title,
                    &entry.lore_type,
                    Some(&path_str),
                    Some(&metadata_str),
                    Some(&related_str),
                ) {
                    Ok(id) => {
                        imported += 1;
                        println!("  ✅ [{}] {} (type: {}, cards: {})", id, entry.title, entry.lore_type, entry.related_cards.len());
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("  ❌ Fehler beim Einfügen von '{}': {}", entry.title, e);
                    }
                }
            }
            Ok(None) => {
                // File without valid content — skip silently
                skipped += 1;
            }
            Err(e) => {
                failed += 1;
                eprintln!("  ⚠️  Fehler beim Parsen von {}: {}", file_path.display(), e);
            }
        }
    }

    println!();
    println!("📊 Import-Report:");
    println!("   ✅ Importiert: {}", imported);
    println!("   ⏭️  Übersprungen: {}", skipped);
    println!("   ❌ Fehlgeschlagen: {}", failed);
    println!();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("mtg_import={}", log_level))
        .with_target(false)
        .init();

    // Database — default to top-level assets/ folder
    let default_db = {
        // Resolve relative to binary location or CWD
        let cwd = std::env::current_dir().ok()
            .unwrap_or_else(|| PathBuf::from("."));
        // If running from src-tauri/, go up one level to project root
        let project_root = if cwd.ends_with("src-tauri") {
            cwd.parent().unwrap_or(&cwd).to_path_buf()
        } else {
            cwd
        };
        let assets_dir = project_root.join("assets");
        // Ensure assets/ exists
        let _ = std::fs::create_dir_all(&assets_dir);
        assets_dir.join("mtg_multiverse_studio.db")
    };
    let db_path = cli.db.unwrap_or(default_db);
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Fehler beim Öffnen der DB: {}", e);
            std::process::exit(1);
        }
    };

    // Ensure tables exist (run all migrations)
    let migrations = [
        include_str!("../db/migrations/001_initial.sql"),
        include_str!("../db/migrations/002_fts5.sql"),
        include_str!("../db/migrations/003_deck_category.sql"),
        include_str!("../db/migrations/004_collection_ext.sql"),
        include_str!("../db/migrations/005_sets_ext.sql"),
        include_str!("../db/migrations/006_prices.sql"),
    ];
    for sql in &migrations {
        if let Err(e) = conn.execute_batch(sql) {
            eprintln!("❌ Migrations-Error: {}", e);
            std::process::exit(1);
        }
    }

    // Handle Lore import separately
    if matches!(cli.dataset, DatasetArg::Lore) {
        let lore_path = cli.path.unwrap_or_else(|| PathBuf::from("assets/stories/"));
        run_lore_import(&conn, &lore_path, cli.verbose);
        return;
    }

    // HTTP client (only for Scryfall imports)
    let client = reqwest::Client::builder()
        .user_agent("MTGMultiverseStudio/0.1")
        .build()
        .unwrap();

    let dataset: import_engine::Dataset = cli.dataset.into();
    println!("🚀 Starte Import: {:?}", dataset);
    println!("📁 Datenbank: {}", db_path.display());
    println!();

    let reporter = import_engine::ConsoleProgress;

    if dataset == import_engine::Dataset::Sets {
        match import_engine::run_sets_import(&conn, &client, &reporter).await {
            Ok(count) => println!("\n🎉 {} Sets importiert!", count),
            Err(e) => {
                eprintln!("\n❌ Sets-Import fehlgeschlagen: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match import_engine::run_bulk_import(&conn, &client, dataset, &reporter).await {
            Ok(count) => println!("\n🎉 {} Karten importiert!", count),
            Err(e) => {
                eprintln!("\n❌ Import fehlgeschlagen: {}", e);
                std::process::exit(1);
            }
        }
    }
}
