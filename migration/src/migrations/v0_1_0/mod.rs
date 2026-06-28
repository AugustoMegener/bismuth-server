use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm_migration::schema::{boolean, float, text, timestamp};

use crate::{
    entity::{fs_node, shard},
    migrations::v0_1_0::{
        facet_fields::SimpleFacetFieldTableMigration, facets_table::FacetTableMigration,
        fs_addresses_table::FsAddressTableMigration,
        fs_node_generation_table::FsNodeGenerationTableMigration,
        fs_nodes_table::FsNodeTableMigration,
        in_note_addresses_table::InNoteAddressesTableMigration, notes_table::NoteTableMigration,
        shard_addresses_table::ShardAddressTableMigration, shards_table::ShardTableMigration,
    },
    util::{CompoundMigration, mg_insert, mg_table},
};

pub mod facet_fields;
pub mod facets_table;
pub mod fs_addresses_table;
pub mod fs_node_generation_table;
pub mod fs_nodes_table;
pub mod in_note_addresses_table;
pub mod notes_table;
pub mod shard_addresses_table;
pub mod shards_table;

pub fn get() -> CompoundMigration {
    CompoundMigration {
        entries: vec![
            mg_table(FacetTableMigration),
            mg_table(SimpleFacetFieldTableMigration::new("text", text("value"))),
            mg_table(SimpleFacetFieldTableMigration::new(
                "number",
                float("value"),
            )),
            mg_table(SimpleFacetFieldTableMigration::new(
                "toggle",
                boolean("value"),
            )),
            mg_table(SimpleFacetFieldTableMigration::new(
                "timestamp",
                timestamp("value"),
            )),
            mg_table(NoteTableMigration),
            mg_table(InNoteAddressesTableMigration),
            mg_table(FsNodeTableMigration),
            mg_table(FsNodeGenerationTableMigration),
            mg_table(FsAddressTableMigration),
            mg_table(ShardTableMigration),
            mg_insert(shard::ActiveModel {
                name: Set(String::from("Trash")),
                ..Default::default()
            }),
            mg_table(ShardAddressTableMigration),
        ],
    }
}
