use crate::{
    migrations::v0_1_0::{facets_table::Facet, notes_table::Note, shards_table::Shard},
    util::TableMigration,
};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum ShardAddress {
    Table,
    Key,
    ShardKey,
    Name,
    ShardAddressFor,
    FacetKey,
    NoteKey,
}

#[derive(Iden)]
enum ShardAddressFor {
    Enum,
    Facet,
    Note,
}

pub struct ShardAddressTableMigration;

#[async_trait::async_trait]
impl TableMigration for ShardAddressTableMigration {
    fn get_name(&self) -> String {
        ShardAddress::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(integer_null("shard_key"))
            .col(string("name"))
            .col(enumeration(
                "shard_address_for",
                ShardAddressFor::Enum,
                [ShardAddressFor::Facet, ShardAddressFor::Note],
            ))
            .col(integer_uniq("facet_key").null())
            .col(integer_uniq("note_key").null())
            .primary_key(
                Index::create()
                    .unique()
                    .col(ShardAddress::ShardKey)
                    .col(ShardAddress::Name)
                    .col(ShardAddress::ShardAddressFor),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ShardAddress::Table, ShardAddress::ShardKey)
                    .to(Shard::Table, Shard::Key)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ShardAddress::Table, ShardAddress::NoteKey)
                    .to(Note::Table, Note::Key),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ShardAddress::Table, ShardAddress::FacetKey)
                    .to(Facet::Table, Facet::Key),
            )
            .check(
                Expr::col(ShardAddress::ShardAddressFor)
                    .eq("Facet")
                    .and(Expr::col(ShardAddress::FacetKey).is_not_null())
                    .and(Expr::col(ShardAddress::NoteKey).is_null())
                    .or(Expr::col(ShardAddress::ShardAddressFor)
                        .eq("Note")
                        .and(Expr::col(ShardAddress::NoteKey).is_not_null())
                        .and(Expr::col(ShardAddress::FacetKey).is_null())),
            )
    }
}
