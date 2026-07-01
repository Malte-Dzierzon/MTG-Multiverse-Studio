//! Database Module
//! 
//! Provides database connection, schema, and repository access.

pub mod card_repo;
pub mod collection_repo;
pub mod connection;
pub mod deck_repo;
pub mod lore_repo;
pub mod schema;
#[cfg(test)]
mod tests {
    use super::connection::init_test_db;
    
    #[test]
    fn test_db_module_compiles() {
        let _conn = init_test_db().expect("Database should initialize");
    }
}
