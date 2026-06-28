use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::error::Error;
use std::sync::OnceLock;

pub static DATABASE: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init_repo_database(dir_path: &str) -> Result<(), Box<dyn Error>> {
    DATABASE
        .set(Database::connect(format!("sqlite://{dir_path}/.bismuth?mode=rwc")).await?)
        .unwrap();
    let database = DATABASE.get().unwrap();

    Migrator::up(database, None).await?;

    Ok(())
}
