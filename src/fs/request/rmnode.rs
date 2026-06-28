use crate::fs::request::{DBFSErr, FSResult};
use fuser::{Errno, INodeNo};
use migration::entity::{
    fs_node,
    prelude::{FsAdress, FsNode},
};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{DatabaseConnection, ModelTrait};
use std::i64;
use std::sync::Arc;

pub async fn rmnode(
    db_connection: Arc<DatabaseConnection>,
    parent: INodeNo,
    name: String,
    is_dir: bool,
) -> FSResult<()> {
    let node = FsNode::find()
        .filter(fs_node::Column::ParentNodeKey.eq(parent.0 as i64))
        .filter(fs_node::Column::Name.eq(name))
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
        .ok_or(DBFSErr(Errno::ENOENT, None))?;

    let addr_opt = FsAdress::find_by_node_key(node.key)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    if (is_dir && addr_opt.is_some()) || (!is_dir && addr_opt.is_none()) {
        return Err(DBFSErr(Errno::EPERM, None));
    }

    node.delete(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    if let Some(addr) = addr_opt {
        match addr.address_for.as_str() {
            "note" => todo!(),
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}
