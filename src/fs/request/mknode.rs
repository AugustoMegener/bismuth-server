use crate::{
    db,
    fs::request::{
        util::{fs_node_to_attr, new_fs_node_generation},
        DBFSErr, FSResult,
    },
};
use fuser::{Errno, FileAttr, Generation, INodeNo};
use migration::entity::{fs_adress, fs_node};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, TryIntoModel};
use std::sync::Arc;

pub async fn mknode(
    db_connection: Arc<DatabaseConnection>,
    parent: INodeNo,
    name: String,
    user_id: u32,
    group_id: u32,
    is_dir: bool,
    perm: u32,
) -> FSResult<(FileAttr, Generation)> {
    let mut new_node = fs_node::ActiveModel {
        name: Set(name),
        parent_node_key: Set(Some(parent.0 as i64)),
        user_id: Set(user_id as i64),
        group_id: Set(group_id as i64),
        permission_flags: Set(perm as i64),

        ..Default::default()
    }
    .save(&*db_connection)
    .await
    .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let node_key = new_node.key.take().ok_or(DBFSErr(Errno::EIO, None))?;

    let new_address = if is_dir {
        None
    } else {
        Some(
            fs_adress::ActiveModel {
                node_key: Set(node_key),
                address_for: Set("unset_file".to_string()),

                ..Default::default()
            }
            .save(&*db_connection)
            .await
            .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
            .try_into_model()
            .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?,
        )
    };

    let generation = new_fs_node_generation(&*db_connection, node_key).await?;

    let node_model = new_node
        .try_into_model()
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    Ok((fs_node_to_attr(&node_model, new_address)?, generation))
}
