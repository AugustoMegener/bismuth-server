use crate::{migrations::UUID_DEFAULT, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum Note {
    Table,
    Key,
}

#[derive(Iden)]
enum NoteAddressType {
    Enum,
    Filesystem,
    Shard,
}

pub struct NoteTableMigration;

#[async_trait::async_trait]
impl TableMigration for NoteTableMigration {
    fn get_name(&self) -> String {
        Note::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto("key").unique_key())
            .col(text_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(text("content"))
            .col(enumeration(
                "note_address_type",
                NoteAddressType::Enum,
                [NoteAddressType::Filesystem, NoteAddressType::Shard],
            ))
    }
}
