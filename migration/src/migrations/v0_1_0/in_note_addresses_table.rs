use sea_orm_migration::{prelude::{ForeignKey, Index, TableCreateStatement}, schema::*, sea_orm::Iden, * };
use crate::{migrations::v0_1_0::{facets_table::Facets, notes_table::Notes}, util::TableMigration};

#[derive(Iden)]
pub enum InNoteAddresses {
    Table, NoteKey, ColStart, LineStart,  FacetKey
}

pub struct InNoteAddressesTableMigration;

#[async_trait::async_trait]
impl TableMigration for InNoteAddressesTableMigration {
    fn get_name(&self) -> String {
        InNoteAddresses::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(integer("note_key"))
            .col(integer("col_star").default(0))
            .col(integer("line_start"))
            .col(integer_null("col_width"))
            .col(integer("lines_amount").default(1))
            .col(integer_uniq("facet_key")) 
            .primary_key(
                Index::create()
                    .unique()
                    .col(InNoteAddresses::NoteKey)
                    .col(InNoteAddresses::ColStart)
                    .col(InNoteAddresses::LineStart) 
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Notes::Table, Notes::Key)
                    .to(InNoteAddresses::Table, InNoteAddresses::NoteKey)
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Facets::Table, Facets::Key)
                    .to(InNoteAddresses::Table, InNoteAddresses::FacetKey)
            )
    }
}
