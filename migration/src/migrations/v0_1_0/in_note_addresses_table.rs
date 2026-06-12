use crate::{
    migrations::v0_1_0::{facets_table::Facet, notes_table::Note},
    util::TableMigration,
};
use sea_orm_migration::{
    prelude::{ForeignKey, Index, TableCreateStatement},
    schema::*,
    sea_orm::Iden,
    *,
};

#[derive(Iden)]
pub enum InNoteAddress {
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
        InNoteAddress::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(integer(InNoteAddress::NoteKey))
            .col(integer(InNoteAddress::ColStart).default(0))
            .col(integer(InNoteAddress::LineStart))
            .col(integer_null("col_width"))
            .col(integer("lines_amount").default(1))
            .col(integer_uniq(InNoteAddress::FacetKey))
            .primary_key(
                Index::create()
                    .unique()
                    .col(InNoteAddress::NoteKey)
                    .col(InNoteAddress::ColStart)
                    .col(InNoteAddress::LineStart),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(InNoteAddress::Table, InNoteAddress::NoteKey)
                    .to(Note::Table, Note::Key),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(InNoteAddress::Table, InNoteAddress::FacetKey)
                    .to(Facet::Table, Facet::Key),
            )
    }
}
