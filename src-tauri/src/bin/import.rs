//! MTG Multiverse Studio — Bulk Import CLI
//!
//! Command-line tool to import Scryfall card data into the local SQLite database.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin mtg-import -- oracle
//! cargo run --bin mtg-import -- default-cards
//! cargo run --bin mtg-import -- oracle --db /custom/path/mtg.db
//! ```

use clap::Parser;
use std::path::PathBuf;

#[path = "../import_engine/mod.rs"]
mod import_engine;

#[derive(clap::ValueEnum, Debug, Clone)]
enum DatasetArg {
    Oracle,
    #[clap(name = "default-cards")]
    DefaultCards,
    Rulings,
    Sets,
}

impl From<DatasetArg> for import_engine::Dataset {
    fn from(arg: DatasetArg) -> Self {
        match arg {
            DatasetArg::Oracle => import_engine::Dataset::OracleCards,
            DatasetArg::DefaultCards => import_engine::Dataset::DefaultCards,
            DatasetArg::Rulings => import_engine::Dataset::Rulings,
            DatasetArg::Sets => import_engine::Dataset::Sets,
        }
    }
}

/// MTG Multiverse Studio — Scryfall Bulk Import Tool
#[derive(Parser, Debug)]
#[command(name = "mtg-import", version, about)]
struct Cli {
    /// Dataset to import: oracle | default-cards | rulings | sets
    #[arg(value_enum)]
    dataset: DatasetArg,

    /// Path to SQLite database (default: mtg_multiverse_studio.db)
    #[arg(short, long)]
    db: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
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

    // Database
    let db_path = cli.db.unwrap_or_else(|| PathBuf::from("mtg_multiverse_studio.db"));
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
    ];
    for sql in &migrations {
        if let Err(e) = conn.execute_batch(sql) {
            eprintln!("❌ Migrations-Error: {}", e);
            std::process::exit(1);
        }
    }

    // HTTP client
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
