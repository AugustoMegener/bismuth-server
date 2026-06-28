use crate::fs::request::util::get_fsnode_attr;
use crate::fs::request::FSResult;
use fuser::{FileAttr, INodeNo};
use migration::entity::prelude::*;
use sea_orm::DatabaseConnection;
use std::i64;
use std::sync::Arc;

pub async fn getattr(db_connection: Arc<DatabaseConnection>, ino: INodeNo) -> FSResult<FileAttr> {
    Ok(
        get_fsnode_attr(&*db_connection, FsNode::find_by_key(ino.0 as i64))
            .await?
            .0,
    )
}
