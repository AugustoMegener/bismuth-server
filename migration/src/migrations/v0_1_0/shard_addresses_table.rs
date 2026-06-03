use sea_orm_migration::{prelude::{ *}, schema::*};
use crate::{migrations::{UUID_DEFAULT, v0_1_0::{facets_table::Facets, fs_nodes_table::FsNodes, notes_table::Notes, shards_table::Shards}}, util::TableMigration};

#[derive(Iden)]
pub enum ShardAddresses {
    Table, Key, ShardKey, Name, ShardAddressFor, FacetKey, NoteKey
}

#[derive(Iden)]
enum ShardAddressFor {
   Enum, Facet, Note 
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
            .col(enumeration("shard_address_for", ShardAddressFor::Enum, [ShardAddressFor::Facet, ShardAddressFor::Note]))
            .col(integer_uniq("facet_key").null())
            .col(integer_uniq("note_key").null())
            .primary_key(
                Index::create()
                    .unique()
                    .col(ShardAddresses::ShardKey)
                    .col(ShardAddresses::Name)
                    .col(ShardAddresses::ShardAddressFor)
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Shards::Table, Shards::Key)
                    .to(ShardAddresses::Table, ShardAddresses::ShardKey)
                    .on_delete(ForeignKeyAction::Cascade)
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Notes::Table, Notes::Key)
                    .to(ShardAddresses::Table, ShardAddresses::NoteKey)
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Facets::Table, Facets::Key)
                    .to(ShardAddresses::Table, ShardAddresses::FacetKey)
            )
        
    }
}
