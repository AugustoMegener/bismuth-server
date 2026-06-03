use sea_orm_migration::{prelude::{ *}, schema::*};
use crate::{migrations::UUID_DEFAULT, util::TableMigration};

#[derive(Iden)]
pub enum FsNodes {
    Table, Key, ParentNodeKey
}


pub struct FsNodesTableMigration;

#[async_trait::async_trait]
impl TableMigration for FsNodesTableMigration {
    fn get_name(&self) -> String {
        FsNodes::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(uuid_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(string_uniq("name"))
            .col(date_time_null("creation_time").default(Expr::current_time()))
            .col(date_time_null("last_acess"))
            .col(date_time_null("last_modf"))
            .col(date_time_null("last_node_change"))
            .col(integer_null("parent_node_key"))
            .foreign_key(
                ForeignKey::create()
                    .from(FsNodes::Table, FsNodes::Key)
                    .to(FsNodes::Table, FsNodes::ParentNodeKey)
                    .on_delete(ForeignKeyAction::Cascade)
            )
    }
}
