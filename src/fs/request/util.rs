use std::i64;
use std::sync::LazyLock;
use std::time::SystemTime;

use chrono::Utc;
use fuser::FileType;
use fuser::{Errno, FileAttr, Generation, INodeNo};
use migration::entity::prelude::*;
use migration::entity::shard::{self, Entity};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, DbErr, Select};

use crate::fs::request::{DBFSErr, FSResult};
use migration;
use migration::entity::prelude::{FsAdress, FsNode, FsNodeGeneration};
use migration::entity::{fs_adress, fs_node, fs_node_generation};
use sea_orm::prelude::*;

const DIR_SIZE: u64 = 0x1000;

static TRASH_SHARD_NAME: &'static str = "Trash";

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

pub struct AttrPatch {
    pub perm: Option<u32>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub size: Option<u64>,
    pub atime: Option<fuser::TimeOrNow>,
    pub mtime: Option<fuser::TimeOrNow>,
    pub ctime: Option<std::time::SystemTime>,
    pub fh: Option<fuser::FileHandle>,
    pub crtime: Option<std::time::SystemTime>,
    pub chgtime: Option<std::time::SystemTime>,
    pub bkuptime: Option<std::time::SystemTime>,
    pub flags: Option<fuser::BsdFileFlags>,
}

pub fn system_time_to_naive(st: SystemTime) -> DateTime {
    chrono::DateTime::<Utc>::from(st).naive_utc()
}

pub fn fs_node_to_attr(
    node: &fs_node::Model,
    addr: Option<fs_adress::Model>,
) -> FSResult<FileAttr> {
    let file_size = node.size.map(|it| it as u64).unwrap_or(DIR_SIZE);

    let Ok(kind) = FsFiletype::parse(addr.as_ref().map(|it| it.address_for.as_str())) else {
        return Err(DBFSErr(Errno::EIO, None));
    };

    let into_system_time = |it: Option<DateTime>| it.unwrap_or_default().and_utc().into();

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

pub async fn get_fsnode_attr(
    db_connection: &DatabaseConnection,
    fs_node_select: Select<FsNode>,
) -> Result<(FileAttr, fs_node::Model), DBFSErr> {
    let node = fs_node_select
        .one(db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
        .ok_or(DBFSErr(Errno::ENOENT, None))?;

    let addr = FsAdress::find_by_node_key(node.key)
        .one(&*db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    Ok((fs_node_to_attr(&node, addr)?, node))
}

pub async fn get_fs_node_generation(
    db_connection: &DatabaseConnection,
    fs_node_key: i64,
) -> Result<Generation, DBFSErr> {
    let gen_query = FsNodeGeneration::find_by_node_key(fs_node_key)
        .one(db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let gen = match gen_query {
        Some(row) => row.generation,
        None => {
            let inserted = fs_node_generation::ActiveModel {
                node_key: Set(fs_node_key),
                ..Default::default()
            }
            .insert(&*db_connection)
            .await
            .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

            inserted.generation
        }
    };

    Ok(Generation(gen as u64))
}

pub async fn new_fs_node_generation(
    db_connection: &DatabaseConnection,
    fs_node_key: i64,
) -> Result<Generation, DBFSErr> {
    let gen_query = FsNodeGeneration::find_by_node_key(fs_node_key)
        .one(db_connection)
        .await
        .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

    let gen = match gen_query {
        Some(row) => {
            let next_gen = row.generation + 1;
            let mut active: fs_node_generation::ActiveModel = row.into();

            active.generation = Set(next_gen);

            active
                .save(db_connection)
                .await
                .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?
                .generation
                .take()
                .ok_or(DBFSErr(Errno::EIO, None))?
        }
        None => {
            let inserted = fs_node_generation::ActiveModel {
                node_key: Set(fs_node_key),
                ..Default::default()
            }
            .insert(&*db_connection)
            .await
            .map_err(|error| DBFSErr(Errno::EIO, Some(error)))?;

            inserted.generation
        }
    };

    Ok(Generation(gen as u64))
}

pub async fn get_trash_shard(db_connection: &DatabaseConnection) -> Result<shard::Model, DbErr> {
    let existing = Shard::find()
        .filter(shard::Column::ParentShardKey.eq(None::<i64>))
        .filter(shard::Column::Name.eq(String::from(TRASH_SHARD_NAME)))
        .one(db_connection)
        .await?;

    if let Some(shard) = existing {
        return Ok(shard);
    }

    shard::ActiveModel {
        name: Set(String::from(TRASH_SHARD_NAME)),
        ..Default::default()
    }
    .insert(db_connection)
    .await
}
