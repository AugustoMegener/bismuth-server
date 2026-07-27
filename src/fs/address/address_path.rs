use migration::entity::note::{self, Entity};

use migration::entity::{
    facet, fs_adress, fs_node, in_note_address, prelude::*, shard, shard_address,
};
use migration::prelude::serde_json;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter, QuerySelect,
    Statement,
};
use sea_orm::{EntityName, FromQueryResult, Iden};

use crate::fs::address::address_owner::AddressOwner;
use crate::fs::address::find_one;
use crate::fs::address::resolved_address::{ResolvedAddress, ResolvedAddressTrait};
use sea_orm::sea_query::{Alias, Expr, Query, UnionType};

#[derive(Debug, FromQueryResult)]
struct NoteOrFacet {
    kind: String,
    key: i64,
    address_type: String,
}

enum RawPathOwner {
    Note,
    Shard,
}

pub enum AddressPath {
    RawPath(String),
    ShardPath(String),
    FsPath(String),
    InNotePath(String),
}

impl AddressPath {
    pub fn parse(path: String) -> Option<AddressPath> {
        match path.chars().next() {
            Some('$') if AddressPath::is_raw_path(&path) => Some(AddressPath::RawPath(path)),
            Some('<') if AddressPath::is_fs_path(&path) => Some(AddressPath::FsPath(path)),
            Some('<') if AddressPath::is_shard_path(&path) => Some(AddressPath::ShardPath(path)),
            Some('[') if AddressPath::is_fs_path(&path) => Some(AddressPath::FsPath(path)),
            Some('[') if AddressPath::is_in_note_path(&path) => Some(AddressPath::InNotePath(path)),
            _ => None,
        }
    }

    pub fn is_raw_path(path: &str) -> bool {
        path.starts_with("$(")
            && path.ends_with(')')
            && path.len() == 39
            && path[2..38]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
    }

    pub fn is_shard_path(path: &str) -> bool {
        if !path.starts_with('<') && !path.ends_with('>') {
            return false;
        }
        path[1..path.len() - 1]
            .chars()
            .all(|it| it.is_alphanumeric() || matches!(it, '.' | ':'))
    }

    pub fn is_fs_path(path: &str) -> bool {
        path.chars().next() == Some('[') && path.chars().last() == Some(']')
    }

    pub fn is_in_note_path(path: &str) -> bool {
        let Some(opener) = path.chars().next().filter(|c| matches!(c, '<' | '[')) else {
            return false;
        };
        if path.chars().last() == Some(')') {
            return false;
        }

        let closer = match opener {
            '<' => '>',
            '[' => ']',
            _ => return false,
        };

        let Some(addr_close) = path.find(closer) else {
            return false;
        };

        if path.as_bytes().get(addr_close + 1) != Some(&b'(') {
            return false;
        }

        if !match opener {
            '<' => AddressPath::is_shard_path(&path[..addr_close]),
            '[' => AddressPath::is_fs_path(&path[..addr_close]),
            _ => false,
        } {
            return false;
        }

        let pos = path[addr_close + 2..path.len() - 1].replace(' ', "");
        let Some((row, column)) = pos.split_once(';') else {
            return false;
        };

        (!row.is_empty() && row.chars().all(|it| it.is_ascii_digit()))
            && (column == "~"
                || (!column.is_empty() && column.chars().all(|it| it.is_ascii_digit())))
    }

    pub async fn resolve(
        self,
        db_connection: &DatabaseConnection,
    ) -> Result<Option<ResolvedAddress>, DbErr> {
        match self {
            AddressPath::RawPath(path) => {
                let uuid = &path[2..path.len() - 1];

                let note_query = Query::select()
                    .expr_as(Expr::val("note"), Alias::new("kind"))
                    .column(note::Column::Key)
                    .column(note::Column::AddressType)
                    .from(note::Entity)
                    .and_where(note::Column::Key.eq(uuid))
                    .to_owned();

                let mut union_query = Query::select()
                    .expr_as(Expr::val("facet"), Alias::new("kind"))
                    .column(facet::Column::Key)
                    .column(facet::Column::AddressType)
                    .from(facet::Entity)
                    .and_where(facet::Column::Key.eq(uuid))
                    .to_owned();

                union_query.union(UnionType::All, note_query);

                let builder = db_connection.get_database_backend();
                let Some(found) = NoteOrFacet::find_by_statement(builder.build(&union_query))
                    .one(db_connection)
                    .await?
                else {
                    return Ok(None);
                };

                match (found.kind.as_str(), found.address_type.as_str()) {
                    ("note", "filesystem") => {
                        find_one(
                            FsAdress::find_by_note_key(found.key),
                            db_connection,
                            ResolvedAddress::FsAddr,
                        )
                        .await
                    }
                    ("note", "shard") => {
                        find_one(
                            ShardAddress::find_by_note_key(found.key),
                            db_connection,
                            ResolvedAddress::ShardAddr,
                        )
                        .await
                    }
                    ("facet", "in_note") => {
                        find_one(
                            InNoteAddress::find_by_facet_key(found.key),
                            db_connection,
                            ResolvedAddress::InNoteAddr,
                        )
                        .await
                    }
                    ("facet", "shard") => {
                        find_one(
                            ShardAddress::find_by_facet_key(found.key),
                            db_connection,
                            ResolvedAddress::ShardAddr,
                        )
                        .await
                    }
                    (_, other) => Err(DbErr::Custom(format!("unknown address_for: {other}"))),
                }
            }

            AddressPath::ShardPath(path) => {
                let Some((shard_path, name)) = path[1..path.len() - 1].split_once(':') else {
                    return Ok(None);
                };
                let path_json = serde_json::to_string(&shard_path.split('.').collect::<Vec<_>>())
                    .map_err(|it| DbErr::Custom(it.to_string()))?;

                let shard_table = shard::Entity.table_name();
                let shard_key_col = shard::Column::Key.to_string();
                let shard_name_col = shard::Column::Name.to_string();
                let shard_parent_col = shard::Column::ParentShardKey.to_string();

                let addr_table = shard_address::Entity.table_name();
                let addr_shard_key_col = shard_address::Column::ShardKey.to_string();
                let addr_name_col = shard_address::Column::Name.to_string();

                let query = format!(
                    r#"
                    WITH RECURSIVE segs AS (
                        SELECT key AS idx, value AS name FROM json_each(?1)
                    ),
                    walk AS (
                        SELECT s.*, 0 AS idx
                        FROM {shard_table} s
                        JOIN segs seg ON seg.idx = 0 AND seg.name = s.{shard_name_col}
                        WHERE s.{shard_parent_col} IS NULL

                        UNION ALL

                        SELECT s.*, w.idx + 1 AS idx
                        FROM {shard_table} s
                        JOIN walk w ON s.{shard_parent_col} = w.{shard_key_col}
                        JOIN segs seg ON seg.idx = w.idx + 1 AND seg.name = s.{shard_name_col}
                    )
                    SELECT sa.*
                    FROM walk w
                    JOIN {addr_table} sa ON sa.{addr_shard_key_col} = w.{shard_key_col} AND sa.{addr_name_col} = ?2
                    WHERE w.idx = (SELECT MAX(idx) FROM segs)
                    ORDER BY w.idx DESC
                    LIMIT 1
                    "#
                );

                Ok(shard_address::Entity::find()
                    .from_raw_sql(Statement::from_sql_and_values(
                        db_connection.get_database_backend(),
                        &query,
                        [path_json.into(), name.into()],
                    ))
                    .one(db_connection)
                    .await?
                    .map(ResolvedAddress::ShardAddr))
            }
            AddressPath::FsPath(path) => {
                let path_json =
                    serde_json::to_string(&path[1..path.len()].split('/').collect::<Vec<_>>())
                        .map_err(|it| DbErr::Custom(it.to_string()))?;

                let fs_node_table = fs_node::Entity.table_name();
                let fs_node_key_col = fs_node::Column::Key.to_string();
                let fs_node_name_col = fs_node::Column::Name.to_string();
                let fs_node_parent_col = fs_node::Column::ParentNodeKey.to_string();

                let addr_table = fs_adress::Entity.table_name();
                let addr_node_key_col = fs_adress::Column::NodeKey.to_string();

                let query = format!(
                    r#"
                    WITH RECURSIVE segs AS (
                        SELECT key AS idx, value AS name FROM json_each(?1)
                    ),
                    walk AS (
                        SELECT s.*, 0 AS idx
                        FROM {fs_node_table} s
                        JOIN segs seg ON seg.idx = 0 AND seg.name = s.{fs_node_name_col}
                        WHERE s.{fs_node_parent_col} IS NULL

                        UNION ALL

                        SELECT s.*, w.idx + 1 AS idx
                        FROM {fs_node_table} s
                        JOIN walk w ON s.{fs_node_parent_col} = w.{fs_node_key_col}
                        JOIN segs seg ON seg.idx = w.idx + 1 AND seg.name = s.{fs_node_name_col}
                    )
                    SELECT sa.*
                    FROM walk w
                    JOIN {addr_table} sa ON sa.{addr_node_key_col} = w.{fs_node_key_col}
                    WHERE w.idx = (SELECT MAX(idx) FROM segs)
                    ORDER BY w.idx DESC
                    LIMIT 1
                "#
                );

                Ok(fs_adress::Entity::find()
                    .from_raw_sql(Statement::from_sql_and_values(
                        db_connection.get_database_backend(),
                        &query,
                        [path_json.into()],
                    ))
                    .one(db_connection)
                    .await?
                    .map(ResolvedAddress::FsAddr))
            }
            AddressPath::InNotePath(path) => {
                let Some((path, pos)) = path[..path.len() - 1].split_once('(') else {
                    return Ok(None);
                };

                let pos = pos.replace(' ', "");

                let Some((line, column)) = pos.split_once(';') else {
                    return Ok(None);
                };

                let Some(note_addr) = Box::pin(
                    match path.chars().next() {
                        Some('[') => AddressPath::FsPath(path.to_string()),
                        Some('<') => AddressPath::ShardPath(path.to_string()),
                        _ => return Ok(None),
                    }
                    .resolve(db_connection),
                )
                .await?
                else {
                    return Ok(None);
                };

                let Some(AddressOwner::Note(note)) = note_addr.get_owner(db_connection).await?
                else {
                    return Ok(None);
                };

                Ok(find_one(
                    InNoteAddress::find()
                        .filter(in_note_address::Column::NoteKey.eq(note.key))
                        .filter(
                            in_note_address::Column::LineStart.eq(line
                                .parse::<i64>()
                                .map_err(|it| DbErr::Custom(it.to_string()))?),
                        )
                        .filter(
                            in_note_address::Column::ColStart.eq(match column {
                                "~" => None,
                                it => Some(
                                    it.parse::<i64>()
                                        .map_err(|it| DbErr::Custom(it.to_string()))?,
                                ),
                            }),
                        ),
                    db_connection,
                    ResolvedAddress::InNoteAddr,
                )
                .await?)
            }
        }
    }
}
