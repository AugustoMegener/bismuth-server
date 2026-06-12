use crate::{migrations::v0_1_0::fs_nodes_table::FsNode, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum FsAdress {
    Table,
    NodeKey,
    Extension,
    FsAddressFor,
    RawFileData,
    NoteKey,
}

#[derive(Iden)]
enum FsAddressFor {
    Enum,
    RawFile,
    Note,
    Symlink,
}

pub struct FsAddressTableMigration;

#[async_trait::async_trait]
impl TableMigration for FsAddressTableMigration {
    fn get_name(&self) -> String {
        FsAdress::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(integer("node_key").unique_key())
            .col(text("extension"))
            .col(enumeration(
                "fs_address_for",
                FsAddressFor::Enum,
                [
                    FsAddressFor::RawFile,
                    FsAddressFor::Note,
                    FsAddressFor::Symlink,
                ],
            ))
            .col(blob_null("raw_file_data"))
            .col(integer_uniq("note_key").null())
            .primary_key(
                Index::create()
                    .unique()
                    .col(FsAdress::NodeKey)
                    .col(FsAdress::Extension),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(FsAdress::Table, FsAdress::NodeKey)
                    .to(FsNode::Table, FsNode::Key),
            )
            .check(
                Expr::col(FsAdress::FsAddressFor)
                    .eq(FsAddressFor::RawFile.unquoted())
                    .and(Expr::col(FsAdress::NoteKey).is_null())
                    .and(Expr::col(FsAdress::RawFileData).is_not_null())
                    .or(Expr::col(FsAdress::FsAddressFor)
                        .eq(FsAddressFor::Note.unquoted())
                        .and(Expr::col(FsAdress::NoteKey).is_not_null())
                        .and(Expr::col(FsAdress::RawFileData).is_null())),
            )
    }
}
