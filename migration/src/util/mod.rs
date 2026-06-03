use std::alloc::GlobalAlloc;

use sea_orm_migration::{prelude::{ *}, schema::*};


pub trait TableMigration: Send + Sync {
    fn get_name(&self) -> String;
    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement;
}

pub enum MigrationEntry {
    Table(Box<dyn TableMigration + 'static>)
}

pub fn mg_table(table: impl TableMigration + 'static) -> MigrationEntry {
    MigrationEntry::Table(Box::new(table))
}

#[derive(DeriveMigrationName)]
pub struct CompoundMigration {
    pub entries: Vec<MigrationEntry>,
}

#[async_trait::async_trait]
impl MigrationTrait for CompoundMigration {

    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for entry in &self.entries {
            match entry {
                MigrationEntry::Table(table) => {
                    manager.create_table(
                        table.def_table(Table::create().if_not_exists().table(table.get_name())).to_owned()
                    ).await?;
                }
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for entry in self.entries.iter().rev() {
            match entry {
                MigrationEntry::Table(table) => {
                    manager.drop_table(Table::drop().table(table.get_name()).to_owned()).await?;
                }
            }
        }
        Ok(())
    }
}
