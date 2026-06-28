use crate::fs::request::getattr::getattr;
use crate::fs::request::lookup::lookup;
use crate::fs::request::mknode::mknode;
use crate::fs::request::readlink::readlink;
use crate::fs::request::rmnode::rmnode;
use crate::fs::request::setattr::setattr;
use crate::fs::request::util::AttrPatch;
use fuser::{Errno, FileAttr, Generation, INodeNo};
use sea_orm::{DatabaseConnection, DbErr};
use std::sync::{mpsc, Arc};
use tokio::sync::oneshot;
use tokio::task;

mod getattr;
mod lookup;
mod mknode;
mod readlink;
mod rmnode;
mod setattr;
pub mod util;

pub struct DBFSErr(pub Errno, pub Option<DbErr>);

pub type FSResult<T> = Result<T, DBFSErr>;

pub enum BismuthFSRequest {
    Lookup {
        parent: INodeNo,
        name: String,
        response: oneshot::Sender<FSResult<(FileAttr, Generation)>>,
    },
    GetAttr {
        ino: INodeNo,
        response: oneshot::Sender<FSResult<FileAttr>>,
    },
    SetAttr {
        ino: INodeNo,
        patch: AttrPatch,
        response: oneshot::Sender<FSResult<FileAttr>>,
    },
    ReadLink {
        ino: INodeNo,
        response: oneshot::Sender<FSResult<Vec<u8>>>,
    },
    MkNode {
        parent: INodeNo,
        name: String,
        user_id: u32,
        group_id: u32,
        perm: u32,
        is_dir: bool,
        response: oneshot::Sender<FSResult<(FileAttr, Generation)>>,
    },
    RmNode {
        parent: INodeNo,
        name: String,
        is_dir: bool,
        response: oneshot::Sender<FSResult<()>>,
    },
}

async fn bismuth_fs_worker(
    receiver: mpsc::Receiver<BismuthFSRequest>,
    db_connection: Arc<DatabaseConnection>,
) {
    while let Ok(request) = receiver.recv() {
        let db = db_connection.clone();

        task::spawn(async move {
            match request {
                BismuthFSRequest::Lookup {
                    parent,
                    name,
                    response,
                } => {
                    let _ = response.send(lookup(db, parent, name).await);
                }
                BismuthFSRequest::GetAttr { ino, response } => {
                    let _ = response.send(getattr(db, ino).await);
                }
                BismuthFSRequest::SetAttr {
                    ino,
                    patch,
                    response,
                } => {
                    let _ = response.send(setattr(db, ino, patch).await);
                }
                BismuthFSRequest::ReadLink { ino, response } => {
                    let _ = response.send(readlink(db, ino).await);
                }
                BismuthFSRequest::MkNode {
                    parent,
                    name,
                    user_id,
                    group_id,
                    perm,
                    is_dir,
                    response,
                } => {
                    let _ = response
                        .send(mknode(db, parent, name, user_id, group_id, is_dir, perm).await);
                }
                BismuthFSRequest::RmNode {
                    parent,
                    name,
                    is_dir,
                    response,
                } => {
                    let _ = response.send(rmnode(db, parent, name, is_dir).await);
                }
            }
        });
    }
}
