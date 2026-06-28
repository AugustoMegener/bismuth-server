use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, EntityTrait, IntoActiveModel,
};
use std::future::Future;
use std::pin::Pin;

pub trait TableMigration: Send + Sync {
    fn get_name(&self) -> String;
    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement;
}

type BoxFut<'a> = Pin<Box<dyn Future<Output = Result<(), DbErr>> + Send + 'a>>;
type InsertFn = Box<dyn for<'a> Fn(&'a SchemaManagerConnection) -> BoxFut<'a> + Send + Sync>;

pub struct InsertEntry {
    up: InsertFn,
    down: InsertFn,
}

pub enum MigrationEntry {
    Table(Box<dyn TableMigration + 'static>),
    Insert(InsertEntry),
}

pub fn mg_table(table: impl TableMigration + 'static) -> MigrationEntry {
    MigrationEntry::Table(Box::new(table))
}

pub fn mg_insert<A>(model: A) -> MigrationEntry
where
    A: ActiveModelTrait + ActiveModelBehavior + Clone + Send + Sync + 'static,
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
{
    let model_up = model.clone();
    let model_down = model;

    MigrationEntry::Insert(InsertEntry {
        up: Box::new(move |db| {
            let m = model_up.clone();
            Box::pin(async move {
                m.insert(db).await?;
                Ok(())
            })
        }),
        down: Box::new(move |db| {
            let m = model_down.clone();
            Box::pin(async move {
                m.delete(db).await?;
                Ok(())
            })
        }),
    })
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
                    eprintln!(">>> creating: {}", table.get_name());
                    manager
                        .create_table(
                            table
                                .def_table(Table::create().if_not_exists().table(table.get_name()))
                                .to_owned(),
                        )
                        .await?;
                }
                MigrationEntry::Insert(entry) => {
                    (entry.up)(manager.get_connection()).await?;
                }
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for entry in self.entries.iter().rev() {
            match entry {
                MigrationEntry::Table(table) => {
                    manager
                        .drop_table(Table::drop().table(table.get_name()).to_owned())
                        .await?;
                }
                MigrationEntry::Insert(entry) => {
                    (entry.down)(manager.get_connection()).await?;
                }
            }
        }
        Ok(())
    }
}
