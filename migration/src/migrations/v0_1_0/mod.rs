use sea_orm_migration::schema::{boolean, float, text, timestamp};

use crate::{
    migrations::v0_1_0::{
        facet_fields::SimpleFacetFieldTableMigration, facets_table::FacetsTableMigration, fs_addresses_table::FsAddressTableMigration, fs_nodes_table::FsNodesTableMigration, in_note_addresses_table::InNoteAddressesTableMigration, notes_table::NotesTableMigration, shard_addresses_table::ShardAddressesTableMigration, shards_table::ShardTableMigration
    },
    util::{CompoundMigration, mg_table}
};

pub mod fs_nodes_table;
pub mod fs_addresses_table;
pub mod notes_table;
pub mod shards_table;
pub mod shard_addresses_table;
pub mod facets_table;
pub mod in_note_addresses_table;
pub mod facet_fields;


pub fn get() -> CompoundMigration {
    CompoundMigration {
        entries: vec![
            mg_table(FacetsTableMigration),
            mg_table(SimpleFacetFieldTableMigration::new("text", text("value"))),
						mg_table(SimpleFacetFieldTableMigration::new("number", float("value"))),
						mg_table(SimpleFacetFieldTableMigration::new("toggle", boolean("value"))),
						mg_table(SimpleFacetFieldTableMigration::new("timestamp", timestamp("value"))),

            mg_table(NotesTableMigration),
            mg_table(InNoteAddressesTableMigration),
            mg_table(FsNodesTableMigration),
            mg_table(FsAddressTableMigration),
            mg_table(ShardTableMigration),
            mg_table(ShardAddressesTableMigration),


        ]
    }
}
