//! MTG Multiverse Studio - Main Entry Point
//! 
//! Tauri v2 application entry point with database initialization and command registration.

mod db;
mod market;
mod models;
mod scryfall;
mod services;
mod utils;
mod commands;
mod import_engine;

use tauri::Manager;
use std::sync::Mutex;
use crate::market::cardmarket::CardmarketClient;
use crate::market::tcgplayer::TcgplayerClient;

/// Application state shared across commands
struct AppState {
    db: Mutex<rusqlite::Connection>,
    scryfall_client: Mutex<scryfall::client::ScryfallClient>,
    cardmarket_client: Mutex<CardmarketClient>,
    tcgplayer_client: Mutex<TcgplayerClient>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let db = db::connection::init_db(app.handle())
                .expect("Failed to initialize database");
            
            // Initialize Scryfall client
            let scryfall_client = scryfall::client::ScryfallClient::new();
            
            // Initialize market API clients (they gracefully handle missing env vars)
            let cardmarket_client = CardmarketClient::new();
            let tcgplayer_client = TcgplayerClient::new();

            // Manage app state
            app.manage(AppState {
                db: Mutex::new(db),
                scryfall_client: Mutex::new(scryfall_client),
                cardmarket_client: Mutex::new(cardmarket_client),
                tcgplayer_client: Mutex::new(tcgplayer_client),
            });
            
            tracing::info!("MTG Multiverse Studio initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_cards,
            commands::get_card,
            commands::add_to_collection,
            commands::create_deck,
            commands::get_deck,
            commands::list_decks,
            commands::add_card_to_deck,
            commands::remove_card_from_deck,
            commands::update_deck,
            commands::update_deck_card_quantity,
            commands::reorder_deck_cards,
            commands::delete_deck,
            commands::search_decks,
            commands::validate_deck,
            commands::check_deck_legality,
            commands::goldfish_deck,
            commands::load_lore_entries,
            commands::get_lore_entry,
            commands::get_lore_content,
            commands::search_lore,
            commands::get_deck_mana_curve,
            commands::get_collection,
            commands::search_collection,
            commands::update_collection_item,
            commands::remove_from_collection,
            commands::import_collection,
            commands::list_sets,
            commands::get_set,
            commands::refresh_prices,
            commands::get_card_prices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}