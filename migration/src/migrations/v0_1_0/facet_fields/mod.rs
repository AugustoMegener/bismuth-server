use sea_orm_migration::{prelude::{Alias, ForeignKey, Index, IntoColumnDef, TableCreateStatement }, schema::{integer, pk_auto, text}};

use crate::{migrations::v0_1_0::facets_table::Facets, util::TableMigration};


pub trait FacetFieldTableMigration {
    fn get_field_name(&self) -> &'static str;

    fn def_field_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement;

}


impl<T: FacetFieldTableMigration + Send + Sync + 'static> TableMigration for T {
    fn get_name(&self) -> String {
        format!("{}_facet_field", self.get_field_name())
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        self.def_field_table(
            table
            .col(integer("facet_key"))
            .col(text("name"))
            .primary_key(
                Index::create()
                .unique()
                .col(Alias::new("facet_key"))
                .col(Alias::new("name"))
            )
            .foreign_key(
                ForeignKey::create()
                .from(Facets::Table, Facets::Key)
                .from(Alias::new(self.get_name()), Alias::new("facet_key"))
            )

        )
    }
}

pub struct SimpleFacetFieldTableMigration<T : IntoColumnDef + Clone> {
    name: &'static str,
    column: T 
}

impl<T : IntoColumnDef + Clone> FacetFieldTableMigration for SimpleFacetFieldTableMigration<T> {
    fn get_field_name(&self) -> &'static str {
        self.name
    }

    fn def_field_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table.col(self.column.clone())
    }
}

impl<T : IntoColumnDef + Clone> SimpleFacetFieldTableMigration<T> {
    pub fn new(name: &'static str, column: T) -> SimpleFacetFieldTableMigration<T> {
        SimpleFacetFieldTableMigration { name, column }
    }
}
