use crate::fs::request::util::{fs_node_to_attr, get_fsnode_attr, system_time_to_naive, AttrPatch};
use crate::fs::request::{DBFSErr, FSResult};
use fuser::{Errno, FileAttr, INodeNo};
use migration::entity::{fs_node, prelude::*};
use sea_orm::prelude::DateTime;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use std::error::Error;
use std::i64;
use std::sync::Arc;
use std::time::SystemTime;

pub async fn setattr(
    db_connection: Arc<DatabaseConnection>,
    ino: INodeNo,
    patch: AttrPatch,
) -> FSResult<FileAttr> {
    let mut active_node: fs_node::ActiveModel = FsNode::find_by_key(ino.0 as i64)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
        .ok_or(DBFSErr(Errno::ENOENT, None))?
        .into();

    if let Some(size) = patch.size {
        active_node.size = Set(Some(size as i64));
    }
    if let Some(ctime) = patch.ctime {
        active_node.creation_time = Set(system_time_to_naive(ctime));
    }
    if let Some(atime) = patch.atime {
        active_node.last_acess = Set(Some(system_time_to_naive(match atime {
            fuser::TimeOrNow::SpecificTime(system_time) => system_time,
            fuser::TimeOrNow::Now => SystemTime::now(),
        })));
    }
    if let Some(mtime) = patch.mtime {
        active_node.last_modf = Set(Some(system_time_to_naive(match mtime {
            fuser::TimeOrNow::SpecificTime(system_time) => system_time,
            fuser::TimeOrNow::Now => SystemTime::now(),
        })));
    }
    if let Some(ctime) = patch.ctime {
        active_node.last_node_change = Set(Some(system_time_to_naive(ctime)));
    }
    if let Some(perm) = patch.perm {
        active_node.permission_flags = Set(perm as i64);
    }
    if let Some(uid) = patch.uid {
        active_node.user_id = Set(uid as i64);
    }
    if let Some(gid) = patch.gid {
        active_node.group_id = Set(gid as i64);
    }
    if let Some(flags) = patch.flags {
        active_node.bsd_flags = Set(flags.bits() as i64);
    }

    let node = active_node
        .update(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let addr = FsAdress::find_by_node_key(node.key)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    Ok(fs_node_to_attr(&node, addr)?)
}
