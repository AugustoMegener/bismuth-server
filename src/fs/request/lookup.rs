use crate::db::entity::{fs_node, fs_node_generation, prelude::*};
use crate::fs::request::util::FsFiletype;
use fuser::{Errno, FileAttr, Generation, INodeNo};
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use std::i64;
use std::sync::Arc;

use crate::fs::request::{DBFSErr, FSResult};

pub async fn lookup(
    db_connection: Arc<DatabaseConnection>,
    parent: INodeNo,
    name: String,
) -> FSResult<(FileAttr, Generation)> {
    let (base_name, _) = name.rsplit_once('.').unwrap_or((&name, ""));

    let node = FsNode::find()
        .filter(fs_node::Column::ParentNodeKey.eq(parent.0 as i64))
        .filter(fs_node::Column::Name.eq(base_name))
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
        .ok_or(DBFSErr(Errno::ENOENT, None))?;

    let node_key = node.key;

    let gen_query = FsNodeGeneration::find_by_node_key(node_key)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let gen = match gen_query {
        Some(row) => row.generation,
        None => {
            let inserted = fs_node_generation::ActiveModel {
                node_key: Set(node_key),
                ..Default::default()
            }
            .insert(&*db_connection)
            .await
            .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

            inserted.generation
        }
    };

    let addr = FsAdress::find_by_node_key(node.key)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let file_size = node.size.unwrap_or(0x1000) as u64;

    let into_system_time = |it: Option<DateTime>| it.unwrap_or_default().and_utc().into();

    let Ok(kind) = FsFiletype::parse(addr.as_ref().map(|it| it.fs_address_for.as_str())) else {
        return Err(DBFSErr(Errno::EIO, None));
    };

    Ok((
        FileAttr {
            ino: INodeNo(node.key as u64),
            size: file_size,
            blocks: ((file_size as f64 / 512.0).ceil()) as u64,
            atime: into_system_time(node.last_acess),
            mtime: into_system_time(node.last_modf),
            ctime: into_system_time(node.last_node_change),
            crtime: node.creation_time.and_utc().into(),
            kind: kind.as_fuse(),
            perm: node.permission_flags as u16,
            nlink: node.hard_link_amount as u32,
            uid: node.user_id as u32,
            gid: node.group_id as u32,
            rdev: 0,
            blksize: 4096,
            flags: node.bsd_flags as u32,
        },
        Generation(gen as u64),
    ))
}
