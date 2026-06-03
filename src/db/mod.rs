use std::error::Error;
use std::sync::OnceLock;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub mod entity;

pub static DATABASE: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init_repo_database(dir_path: &str) -> Result<(), Box<dyn Error>> { 
    DATABASE.set(Database::connect(format!("sqlite://{dir_path}/.bismuth?mode=rwc")).await?).unwrap();
    let database = DATABASE.get().unwrap();

    database.get_schema_registry("bismuth-server::db::entity::*").sync(database).await?;

    Ok(())
}
