use crate::{migrations::UUID_DEFAULT, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum Shards {
    Table,
    Key,
    Name,
    ParentShardKey,
}

pub struct ShardTableMigration;

#[async_trait::async_trait]
impl TableMigration for ShardTableMigration {
    fn get_name(&self) -> String {
        Shards::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto(Shards::Key))
            .col(uuid_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(text(Shards::Name))
            .col(integer_uniq(Shards::ParentShardKey).null())
            .index(
                Index::create()
                    .unique()
                    .col(Shards::Name)
                    .col(Shards::ParentShardKey),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Shards::Table, Shards::ParentShardKey)
                    .to(Shards::Table, Shards::Key)
                    .on_delete(ForeignKeyAction::Cascade),
            )
    }
}
