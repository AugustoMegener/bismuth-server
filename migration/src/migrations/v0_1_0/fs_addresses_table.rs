use sea_orm_migration::{prelude::*, schema::*};
use crate::{migrations::v0_1_0::fs_nodes_table::FsNodes, util::TableMigration};


#[derive(Iden)]
pub enum FsAdresses {
    Table, NodeKey, Extension, FsAddressFor, RawFileData, NoteKey
}

#[derive(Iden)]
enum FsAddressFor {
   Enum, RawFile, Note 
}


pub struct FsAddressTableMigration;

#[async_trait::async_trait]
impl TableMigration for FsAddressTableMigration {
    fn get_name(&self) -> String {
        FsAdresses::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(integer("node_key"))
            .col(text("extension"))
            .col(enumeration("fs_address_for", FsAddressFor::Enum, [FsAddressFor::RawFile, FsAddressFor::Note]))
            .col(blob_null("raw_file_data"))
            .col(integer_uniq("note_key").null())
            .primary_key(
                Index::create()
                    .unique()
                    .col(FsAdresses::NodeKey)
                    .col(FsAdresses::Extension)
            )
            .foreign_key(
                ForeignKey::create()
                    .from(FsNodes::Table, FsNodes::Key)
                    .to(FsAdresses::Table, FsAdresses::NodeKey)
            )
            .check(
                Expr::col(FsAdresses::FsAddressFor)
                    .eq(FsAddressFor::RawFile.unquoted())
                    .and(Expr::col(FsAdresses::NoteKey).is_null())
                    .and(Expr::col(FsAdresses::RawFileData).is_not_null())
                .or(Expr::col(FsAdresses::FsAddressFor)
                    .eq(FsAddressFor::Note.unquoted())
                    .and(Expr::col(FsAdresses::NoteKey).is_not_null())
                    .and(Expr::col(FsAdresses::RawFileData).is_null()))
            )
    }
}
