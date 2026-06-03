use sea_orm_migration::{prelude::{ *}, schema::*};
use crate::{migrations::{UUID_DEFAULT, v0_1_0::{fs_addresses_table::FsAdresses, fs_nodes_table::FsNodes, shard_addresses_table::ShardAddresses}}, util::TableMigration};

#[derive(Iden)]
pub enum Notes {
    Table, Key, 
}

#[derive(Iden)]
enum NoteAddressType {
   Enum, Filesystem, Shard 
}

pub struct NotesTableMigration;

#[async_trait::async_trait]
impl TableMigration for NotesTableMigration {
    fn get_name(&self) -> String {
        Notes::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto("key"))
            .col(uuid_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(text("content"))
            .col(enumeration("notes_address_type", NoteAddressType::Enum, [NoteAddressType::Filesystem, NoteAddressType::Shard]))
    }
}
