use std::sync::Arc;

use fuser::FileType;
use sea_orm::{DatabaseConnection, DbErr};

pub enum FsFiletype {
    Dir,
    RawFile,
    NoteFile,
    Symlink,
}

impl FsFiletype {
    pub fn parse(value: Option<&str>) -> Result<FsFiletype, ()> {
        match value {
            None => Ok(FsFiletype::Dir),
            Some("raw_file") => Ok(FsFiletype::RawFile),
            Some("note") => Ok(FsFiletype::NoteFile),
            Some("symlink") => Ok(FsFiletype::Symlink),
            Some(_) => Err(()),
        }
    }

    pub fn as_fuse(&self) -> FileType {
        match self {
            FsFiletype::Dir => FileType::Directory,
            FsFiletype::RawFile => FileType::RegularFile,
            FsFiletype::NoteFile => FileType::RegularFile,
            FsFiletype::Symlink => FileType::Symlink,
        }
    }
}

fn get_fsnode_attr(db_connection: Arc<DatabaseConnection>) -> Result<FileAttr, DbErr> {
    Ok(FileAtto {})
}
