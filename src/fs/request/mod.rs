use crate::fs::request::getattr::getattr;
use crate::fs::request::lookup::lookup;
use fuser::{Errno, FileAttr, Generation, INodeNo};
use sea_orm::{DatabaseConnection, DbErr};
use std::sync::{mpsc, Arc};
use tokio::sync::oneshot;
use tokio::task;

mod getattr;
mod lookup;
mod util;

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
            }
        });
    }
}
