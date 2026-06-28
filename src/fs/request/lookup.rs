use crate::fs::request::util::{get_fs_node_generation, get_fsnode_attr};
use fuser::{FileAttr, Generation, INodeNo};
use migration::entity::{fs_node, prelude::*};
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use std::i64;
use std::sync::Arc;

use crate::fs::request::FSResult;

pub async fn lookup(
    db_connection: Arc<DatabaseConnection>,
    parent: INodeNo,
    name: String,
) -> FSResult<(FileAttr, Generation)> {
    let (attr, node) = get_fsnode_attr(
        &*db_connection,
        FsNode::find()
            .filter(fs_node::Column::ParentNodeKey.eq(parent.0 as i64))
            .filter(fs_node::Column::Name.eq(name)),
    )
    .await?;

    Ok((
        attr,
        get_fs_node_generation(&*db_connection, node.key).await?,
    ))
}
