//! Market Module - External Price API Integration
//!
//! Provides clients for Cardmarket (MKM) and TCGPlayer APIs
//! to fetch market prices for MTG cards.
//!
//! Both clients gracefully degrade: if no API keys are configured,
//! they return `None` for price queries instead of failing.

pub mod cardmarket;
pub mod tcgplayer;

pub use cardmarket::CardmarketClient;
pub use tcgplayer::TcgplayerClient;
