use crate::db::entity::prelude::*;
use crate::fs::request::util::FsFiletype;
use fuser::{Errno, FileAttr, INodeNo};
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;
use std::i64;
use std::sync::Arc;

use crate::fs::request::{DBFSErr, FSResult};

pub async fn getattr(db_connection: Arc<DatabaseConnection>, ino: INodeNo) -> FSResult<FileAttr> {
    let node = FsNode::find_by_key(ino.0 as i64)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
        .ok_or(DBFSErr(Errno::ENOENT, None))?;

    let addr = FsAdress::find_by_node_key(node.key)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let file_size = node.size.unwrap_or(0x1000) as u64;

    let into_system_time = |it: Option<DateTime>| it.unwrap_or_default().and_utc().into();

    let Ok(kind) = FsFiletype::parse(addr.as_ref().map(|it| it.fs_address_for.as_str())) else {
        return Err(DBFSErr(Errno::EIO, None));
    };

    Ok(FileAttr {
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
    })
}
