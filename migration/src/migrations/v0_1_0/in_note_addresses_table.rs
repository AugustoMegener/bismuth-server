use crate::{
    migrations::v0_1_0::{facets_table::Facets, notes_table::Notes},
    util::TableMigration,
};
use sea_orm_migration::{
    prelude::{ForeignKey, Index, TableCreateStatement},
    schema::*,
    sea_orm::Iden,
    *,
};

#[derive(Iden)]
pub enum InNoteAddresses {
    Table,
    NoteKey,
    ColStart,
    LineStart,
    FacetKey,
}

pub struct InNoteAddressesTableMigration;

#[async_trait::async_trait]
impl TableMigration for InNoteAddressesTableMigration {
    fn get_name(&self) -> String {
        InNoteAddresses::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(integer(InNoteAddresses::NoteKey))
            .col(integer(InNoteAddresses::ColStart).default(0))
            .col(integer(InNoteAddresses::LineStart))
            .col(integer_null("col_width"))
            .col(integer("lines_amount").default(1))
            .col(integer_uniq(InNoteAddresses::FacetKey))
            .primary_key(
                Index::create()
                    .unique()
                    .col(InNoteAddresses::NoteKey)
                    .col(InNoteAddresses::ColStart)
                    .col(InNoteAddresses::LineStart),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(InNoteAddresses::Table, InNoteAddresses::NoteKey)
                    .to(Notes::Table, Notes::Key),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(InNoteAddresses::Table, InNoteAddresses::FacetKey)
                    .to(Facets::Table, Facets::Key),
            )
    }
}
