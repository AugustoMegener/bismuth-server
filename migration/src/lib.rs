pub use sea_orm_migration::prelude::*;

use crate::migrations::v0_1_0;

mod migrations;
pub mod util;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(v0_1_0::get())]
    }
}
