use migration::async_trait::async_trait;
use migration::entity::{
    facet, fs_adress, fs_node, in_note_address, note, prelude::*, shard_address,
};
use sea_orm::sqlx::Encode;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Select};

struct AddressPath(String);

enum AddressPathError {}

enum ResolvedAddress {
    FsAddr(fs_adress::Model),
    ShardAddr(shard_address::Model),
    InNoteAddr(in_note_address::Model),
}

enum AddressOwner {
    Note(note::Model),
    Facet(facet::Model),
    RawFile(fs_node::Model, Vec<u8>),
    UnsetFile(fs_node::Model),
    Symlink(fs_node::Model, ResolvedAddress),
}

impl AddressPath {
    pub fn is_address_path(path: &str) -> bool {
        match path.chars().next() {
            Some('$') => AddressPath::is_raw_path(path),
            Some('<') => AddressPath::is_shard_path(path),
            Some('[') => AddressPath::is_in_fs_path(path),
            _ => false,
        }
    }

    fn is_raw_path(path: &str) -> bool {
        path.starts_with('$')
            && path.len() == 37
            && path[1..]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
    }

    fn is_shard_path(path: &str) -> bool {
        if !path.starts_with('$') {
            return false;
        }
        if let Some(end) = path.find('>') {
            path[..end].chars().all(|c| c.is_alphanumeric() || c == '.')
                && path[end + 1..].starts_with('(')
                && path.ends_with(')')
        } else {
            false
        }
    }

    fn is_in_fs_path(path: &str) -> bool {
        if path.chars().next() != Some('[') {
            return false;
        }

        match path.chars().last() {
            Some(')') => {
                if let Some(addr_close) = path.find("]") {
                    if path.chars().nth(addr_close + 1) != Some('(') {
                        return false;
                    }

                    if let Some((row, column)) =
                        (&path[addr_close + 2..path.size_hint() - 1]).split_once(';')
                    {
                        row.chars().all(|it| it.is_numeric())
                            && (column == "~" || column.chars().all(|it| it.is_numeric()))
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            Some(']') => true,
            _ => false,
        }
    }
}

#[async_trait]
trait ResolvedAddressTrait {
    async fn get_owner(
        &self,
        db_connection: &DatabaseConnection,
    ) -> Result<Option<AddressOwner>, DbErr>;
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
                "symlink" => todo!(),
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

                _ => panic!("unreachable"),
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
        match (self) {
            AddressOwner::Note(note) => match note.note_address_type.as_str() {
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
                _ => panic!("unreachable"),
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
                _ => panic!("unreachable"),
            },
            AddressOwner::RawFile(fs_node, _) | AddressOwner::UnsetFile(fs_node) => {
                find_one(
                    FsAdress::find_by_node_key(fs_node.key),
                    db_connection,
                    ResolvedAddress::FsAddr,
                )
                .await
            }
            AddressOwner::Symlink(fs_node, _) => todo!(),
        }
    }
}

async fn find_one<E, T>(
    select: Select<E>,
    db: &DatabaseConnection,
    map: impl FnOnce(E::Model) -> T,
) -> Result<Option<T>, DbErr>
where
    E: EntityTrait,
{
    Ok(select.one(db).await?.map(map))
}
