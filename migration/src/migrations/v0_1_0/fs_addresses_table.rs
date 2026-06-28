use crate::{migrations::v0_1_0::fs_nodes_table::FsNode, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum FsAdress {
    Table,
    NodeKey,
    AddressFor,
    RawFileData,
    SymlinkPath,
    NoteKey,
}

#[derive(Iden)]
enum FsAddressFor {
    Enum,
    UnsetFile,
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
            .col(integer(FsAdress::NodeKey).unique_key().primary_key())
            .col(enumeration(
                FsAdress::AddressFor,
                FsAddressFor::Enum,
                [
                    FsAddressFor::UnsetFile,
                    FsAddressFor::RawFile,
                    FsAddressFor::Note,
                    FsAddressFor::Symlink,
                ],
            ))
            .col(blob_null(FsAdress::RawFileData))
            .col(integer_uniq(FsAdress::NoteKey).null())
            .col(text_null(FsAdress::SymlinkPath))
            .foreign_key(
                ForeignKey::create()
                    .from(FsAdress::Table, FsAdress::NodeKey)
                    .to(FsNode::Table, FsNode::Key),
            )
            .check(
                Expr::col(FsAdress::AddressFor)
                    .eq(FsAddressFor::RawFile.unquoted())
                    .and(Expr::col(FsAdress::NoteKey).is_null())
                    .and(Expr::col(FsAdress::SymlinkPath).is_null())
                    .and(Expr::col(FsAdress::RawFileData).is_not_null())
                    .or(Expr::col(FsAdress::AddressFor)
                        .eq(FsAddressFor::Note.unquoted())
                        .and(Expr::col(FsAdress::NoteKey).is_not_null())
                        .and(Expr::col(FsAdress::SymlinkPath).is_null())
                        .and(Expr::col(FsAdress::RawFileData).is_null()))
                    .or(Expr::col(FsAdress::AddressFor)
                        .eq(FsAddressFor::Symlink.unquoted())
                        .and(Expr::col(FsAdress::NoteKey).is_null())
                        .and(Expr::col(FsAdress::SymlinkPath).is_not_null())
                        .and(Expr::col(FsAdress::RawFileData).is_null())),
            )
    }
}
