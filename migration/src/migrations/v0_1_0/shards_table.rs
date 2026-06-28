use crate::{migrations::UUID_DEFAULT, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum Shard {
    Table,
    Key,
    Name,
    ParentShardKey,
}

pub struct ShardTableMigration;

#[async_trait::async_trait]
impl TableMigration for ShardTableMigration {
    fn get_name(&self) -> String {
        Shard::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto(Shard::Key).unique_key())
            .col(text_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(text(Shard::Name))
            .col(integer_uniq(Shard::ParentShardKey).null())
            .index(
                Index::create()
                    .unique()
                    .col(Shard::Name)
                    .col(Shard::ParentShardKey),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Shard::Table, Shard::ParentShardKey)
                    .to(Shard::Table, Shard::Key)
                    .on_delete(ForeignKeyAction::Cascade),
            )
    }
}
