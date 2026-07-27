use crate::fs::address::address_owner::AddressOwner;
use crate::fs::address::address_path::AddressPath;
use crate::fs::address::find_one;
use migration::async_trait::async_trait;
use migration::entity::{
    facet, fs_adress, fs_node, in_note_address, note, prelude::*, shard_address,
};
use sea_orm::sqlx::Encode;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Select};

pub enum ResolvedAddress {
    FsAddr(fs_adress::Model),
    ShardAddr(shard_address::Model),
    InNoteAddr(in_note_address::Model),
}

#[async_trait]
pub trait ResolvedAddressTrait {
    async fn get_owner(
        &self,
        db_connection: &DatabaseConnection,
    ) -> Result<Option<AddressOwner>, DbErr>;

    async fn get_path(&self) -> AddressPath;
}

#[async_trait]
impl ResolvedAddressTrait for ResolvedAddress {
    async fn get_owner(
        &self,
        db_connection: &DatabaseConnection,
    ) -> Result<Option<AddressOwner>, DbErr> {
        match self {
            ResolvedAddress::FsAddr(addr) => match addr.address_for.as_str() {
                "unset_file" => {
                    find_one(
                        FsNode::find_by_key(addr.node_key),
                        db_connection,
                        AddressOwner::UnsetFile,
                    )
                    .await
                }

                "raw_file" => {
                    find_one(FsNode::find_by_key(addr.node_key), db_connection, |node| {
                        AddressOwner::RawFile(node, addr.raw_file_data.clone().unwrap_or_default())
                    })
                    .await
                }
                "note" => {
                    if let Some(key) = addr.note_key {
                        find_one(Note::find_by_key(key), db_connection, AddressOwner::Note).await
                    } else {
                        Ok(None)
                    }
                }

                "symlink" => {
                    if let Some(addr_path) = addr.symlink_path.clone().and_then(AddressPath::parse)
                    {
                        find_one(FsNode::find_by_key(addr.node_key), db_connection, |it| {
                            AddressOwner::Symlink(it, addr_path)
                        })
                        .await
                    } else {
                        Ok(None)
                    }
                }
                other => Err(DbErr::Custom(format!("unknown address_for: {other}"))),
            },

            ResolvedAddress::ShardAddr(addr) => match addr.address_for.as_str() {
                "facet" => {
                    if let Some(key) = addr.facet_key {
                        find_one(Facet::find_by_key(key), db_connection, AddressOwner::Facet).await
                    } else {
                        Ok(None)
                    }
                }

                "note" => {
                    if let Some(key) = addr.note_key {
                        find_one(Note::find_by_key(key), db_connection, AddressOwner::Note).await
                    } else {
                        Ok(None)
                    }
                }

                other => Err(DbErr::Custom(format!("unknown address_for: {other}"))),
            },
            ResolvedAddress::InNoteAddr(addr) => {
                find_one(
                    Facet::find_by_key(addr.facet_key),
                    db_connection,
                    AddressOwner::Facet,
                )
                .await
            }
        }
    }

    async fn get_path(&self) -> AddressPath {
        match self {
           FsAdress(addr) => todo!(),
            ShardAddress
        }
    }
}
