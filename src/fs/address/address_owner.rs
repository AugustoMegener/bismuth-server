use migration::async_trait::async_trait;
use migration::entity::prelude::{FsAdress, InNoteAddress, ShardAddress};
use migration::entity::{facet, fs_node, note};
use sea_orm::{DatabaseConnection, DbErr};

use crate::fs::address::address_path::AddressPath;
use crate::fs::address::find_one;
use crate::fs::address::resolved_address::ResolvedAddress;

pub enum AddressOwner {
    Note(note::Model),
    Facet(facet::Model),
    RawFile(fs_node::Model, Vec<u8>),
    UnsetFile(fs_node::Model),
    Symlink(fs_node::Model, AddressPath),
}

#[async_trait]
trait AddressOwnerTrait {
    async fn get_address(
        &self,
        db_connection: &DatabaseConnection,
    ) -> Result<Option<ResolvedAddress>, DbErr>;
}

#[async_trait]
impl AddressOwnerTrait for AddressOwner {
    async fn get_address(
        &self,
        db_connection: &DatabaseConnection,
    ) -> Result<Option<ResolvedAddress>, DbErr> {
        match self {
            AddressOwner::Note(note) => match note.address_type.as_str() {
                "filesystem" => {
                    find_one(FsAdress::find_by_note_key(note.key), db_connection, |it| {
                        ResolvedAddress::FsAddr(it)
                    })
                    .await
                }
                "shard" => {
                    find_one(
                        ShardAddress::find_by_note_key(note.key),
                        db_connection,
                        |it| ResolvedAddress::ShardAddr(it),
                    )
                    .await
                }
                other => Err(DbErr::Custom(format!("unknown address_for: {other}"))),
            },
            AddressOwner::Facet(facet) => match facet.facet_address_type.as_str() {
                "in_note" => {
                    find_one(
                        InNoteAddress::find_by_facet_key(facet.key),
                        db_connection,
                        |it| ResolvedAddress::InNoteAddr(it),
                    )
                    .await
                }
                "shard" => {
                    find_one(
                        ShardAddress::find_by_facet_key(facet.key),
                        db_connection,
                        |it| ResolvedAddress::ShardAddr(it),
                    )
                    .await
                }
                other => Err(DbErr::Custom(format!("unknown address_for: {other}"))),
            },
            AddressOwner::RawFile(fs_node, _) | AddressOwner::UnsetFile(fs_node) => {
                find_one(
                    FsAdress::find_by_node_key(fs_node.key),
                    db_connection,
                    ResolvedAddress::FsAddr,
                )
                .await
            }
            AddressOwner::Symlink(fs_node, _) => {
                find_one(
                    FsAdress::find_by_node_key(fs_node.key),
                    db_connection,
                    ResolvedAddress::FsAddr,
                )
                .await
            }
        }
    }
}
