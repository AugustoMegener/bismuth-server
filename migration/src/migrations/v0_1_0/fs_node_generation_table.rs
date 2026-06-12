use crate::{migrations::v0_1_0::fs_nodes_table::FsNode, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum FsNodeGeneration {
    Table,
    NodeKey,
    Generation,
}

pub struct FsNodeGenerationTableMigration;

#[async_trait::async_trait]
impl TableMigration for FsNodeGenerationTableMigration {
    fn get_name(&self) -> String {
        FsNodeGeneration::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto("node_key").unique_key())
            .col(unsigned("generation").default(0))
    }
}
