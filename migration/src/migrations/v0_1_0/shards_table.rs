use sea_orm_migration::{prelude::{ *}, schema::*};
use crate::{migrations::{UUID_DEFAULT, v0_1_0::fs_nodes_table::FsNodes}, util::TableMigration};

#[derive(Iden)]
pub enum Shards {
    Table, Key, Name, ParentShardKey
}

pub struct ShardTableMigration;

#[async_trait::async_trait]
impl TableMigration for ShardTableMigration {
    fn get_name(&self) -> String {
        Shards::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto("key"))
            .col(uuid_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(text("name"))
            .col(integer_null("parent_shard_key"))
            .index(
                Index::create()
                    .unique()
                    .col(Shards::Name)
                    .col(Shards::ParentShardKey)
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Shards::Table, Shards::Key)
                    .to(Shards::Table, Shards::ParentShardKey)
                    .on_delete(ForeignKeyAction::Cascade)
            )
        
    }
}
