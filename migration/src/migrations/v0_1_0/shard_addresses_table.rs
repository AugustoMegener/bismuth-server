use crate::{
    migrations::v0_1_0::{facets_table::Facets, notes_table::Notes, shards_table::Shards},
    util::TableMigration,
};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum ShardAddresses {
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

pub struct ShardAddressesTableMigration;

#[async_trait::async_trait]
impl TableMigration for ShardAddressesTableMigration {
    fn get_name(&self) -> String {
        ShardAddresses::Table.unquoted().to_string()
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
                    .col(ShardAddresses::ShardKey)
                    .col(ShardAddresses::Name)
                    .col(ShardAddresses::ShardAddressFor),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ShardAddresses::Table, ShardAddresses::ShardKey)
                    .to(Shards::Table, Shards::Key)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ShardAddresses::Table, ShardAddresses::NoteKey)
                    .to(Notes::Table, Notes::Key),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ShardAddresses::Table, ShardAddresses::FacetKey)
                    .to(Facets::Table, Facets::Key),
            )
            .check(
                Expr::col(ShardAddresses::ShardAddressFor)
                    .eq("Facet")
                    .and(Expr::col(ShardAddresses::FacetKey).is_not_null())
                    .and(Expr::col(ShardAddresses::NoteKey).is_null())
                    .or(Expr::col(ShardAddresses::ShardAddressFor)
                        .eq("Note")
                        .and(Expr::col(ShardAddresses::NoteKey).is_not_null())
                        .and(Expr::col(ShardAddresses::FacetKey).is_null())),
            )
    }
}
