use crate::{migrations::UUID_DEFAULT, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum FsNode {
    Table,
    Key,
    Name,
    ParentNodeKey,
}

pub struct FsNodeTableMigration;

#[async_trait::async_trait]
impl TableMigration for FsNodeTableMigration {
    fn get_name(&self) -> String {
        FsNode::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto("key").unique_key())
            .col(text_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(string("name"))
            .col(unsigned_null("size"))
            .col(date_time("creation_time").default(Expr::current_time()))
            .col(date_time_null("last_acess"))
            .col(date_time_null("last_modf"))
            .col(date_time_null("last_node_change"))
            .col(unsigned("permission_flags"))
            .col(unsigned("hard_link_amount").default(0))
            .col(unsigned("user_id"))
            .col(unsigned("group_id"))
            .col(unsigned("bsd_flags").default(0))
            .col(integer_null("parent_node_key"))
            .foreign_key(
                ForeignKey::create()
                    .from(FsNode::Table, FsNode::ParentNodeKey)
                    .to(FsNode::Table, FsNode::Key)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .index(
                Index::create()
                    .unique()
                    .col(FsNode::Name)
                    .col(FsNode::ParentNodeKey),
            )
    }
}
