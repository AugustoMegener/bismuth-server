use crate::fs::request::{DBFSErr, FSResult};
use fuser::{Errno, INodeNo};
use migration::entity::prelude::*;
use sea_orm::DatabaseConnection;
use std::i64;
use std::sync::Arc;

pub async fn readlink(db_connection: Arc<DatabaseConnection>, ino: INodeNo) -> FSResult<Vec<u8>> {
    let symlink = FsAdress::find_by_node_key(ino.0 as i64)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
        .map(|it| it.symlink_path)
        .flatten()
        .ok_or(DBFSErr(Errno::ENOENT, None))?;

    Ok(symlink.into_bytes())
}
