use std::{
    collections::HashMap,
    sync::mpsc::{self},
    time::Duration,
};

use crate::fs::request::util::AttrPatch;
use fuser::{Errno, FileAttr, Filesystem, Generation};
use log::{error, warn};
use sea_orm::ColumnTrait;
use tokio::{
    runtime,
    sync::oneshot::{self, Sender},
    task::block_in_place,
};

use crate::fs::request::{BismuthFSRequest, DBFSErr, FSResult};
use migration::entity::fs_node;

const TTL: Duration = Duration::from_secs(1);

struct BismuthFS {
    sender: mpsc::Sender<BismuthFSRequest>,
}

impl BismuthFS {
    fn send<T, F>(&self, message: F) -> FSResult<T>
    where
        F: FnOnce(Sender<FSResult<T>>) -> BismuthFSRequest,
    {
        let (resp_tx, resp_rx) = oneshot::channel::<FSResult<T>>();

        self.sender
            .clone()
            .send(message(resp_tx))
            .map_err(|_| DBFSErr(Errno::EIO, None))?;

        block_in_place(|| runtime::Handle::current().block_on(resp_rx))
            .map_err(|_| DBFSErr(Errno::EIO, None))?
    }
}

impl Filesystem for BismuthFS {
    fn init(
        &mut self,
        _req: &fuser::Request,
        _config: &mut fuser::KernelConfig,
    ) -> std::io::Result<()> {
        Ok(())
    }

    fn destroy(&mut self) {}

    fn lookup(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let Some(name_string) = name.to_str() else {
            reply.error(Errno::EBADF);
            return;
        };

        match self.send(|tx| BismuthFSRequest::Lookup {
            parent,
            name: name_string.to_string(),
            response: tx,
        }) {
            Ok((attr, gen)) => reply.entry(&TTL, &attr, gen),
            Err(DBFSErr(errno, dberr)) => {
                if let Some(error) = dberr {
                    error!(
                        "Database error trying to lookup INode({})/{}: {error}",
                        parent.0.to_string(),
                        name.to_str().unwrap_or("")
                    );
                }

                reply.error(errno);
            }
        };
    }

    fn getattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        _fh: Option<fuser::FileHandle>,
        reply: fuser::ReplyAttr,
    ) {
        match self.send(|tx| BismuthFSRequest::GetAttr { ino, response: tx }) {
            Ok(attr) => reply.attr(&TTL, &attr),
            Err(DBFSErr(errno, dberr)) => {
                if let Some(error) = dberr {
                    error!(
                        "Database error trying to getattr on INode({}): {error}",
                        ino.0.to_string()
                    );
                }

                reply.error(errno);
            }
        };
    }

    fn setattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        fh: Option<fuser::FileHandle>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        flags: Option<fuser::BsdFileFlags>,
        reply: fuser::ReplyAttr,
    ) {
        match self.send(|tx| BismuthFSRequest::SetAttr {
            ino,
            patch: AttrPatch {
                perm: mode.map(|it| it & 0o7777),
                uid,
                gid,
                size,
                atime: _atime,
                mtime: _mtime,
                ctime: _ctime,
                fh,
                crtime: _crtime,
                chgtime: _chgtime,
                bkuptime: _bkuptime,
                flags,
            },
            response: tx,
        }) {
            Ok(attr) => reply.attr(&TTL, &attr),
            Err(DBFSErr(errno, dberr)) => {
                if let Some(error) = dberr {
                    error!(
                        "Database error trying to setattr on INode({}): {error}",
                        ino.0.to_string()
                    );
                }

                reply.error(errno);
            }
        };
    }

    fn readlink(&self, _req: &fuser::Request, ino: fuser::INodeNo, reply: fuser::ReplyData) {
        match self.send(|tx| BismuthFSRequest::ReadLink { ino, response: tx }) {
            Ok(path) => reply.data(&path),
            Err(DBFSErr(errno, dberr)) => {
                if let Some(error) = dberr {
                    error!(
                        "Database error trying to read symlink of INode({}): {error}",
                        ino.0.to_string()
                    );
                }

                reply.error(errno);
            }
        };
    }

    fn mknod(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: fuser::ReplyEntry,
    ) {
        let _ = rdev;

        let Some(name_string) = name.to_str() else {
            reply.error(Errno::EBADF);
            return;
        };

        match mode & libc::S_IFMT {
            libc::S_IFREG => {
                match self.send(|tx| BismuthFSRequest::MkNode {
                    parent,
                    name: name_string.to_string(),
                    user_id: _req.uid(),
                    group_id: _req.gid(),
                    perm: mode & !umask,
                    is_dir: false,
                    response: tx,
                }) {
                    Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
                    Err(DBFSErr(errno, dberr)) => {
                        if let Some(error) = dberr {
                            error!(
                                "Database error trying to make INode({})/{name_string} node: {error}",
                                parent.0.to_string()
                            );
                        }

                        reply.error(errno);
                    }
                }
            }
            _ => reply.error(Errno::EINVAL),
        }
    }

    fn mkdir(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        let Some(name_string) = name.to_str() else {
            reply.error(Errno::EBADF);
            return;
        };

        match mode & libc::S_IFMT {
            libc::S_IFREG => {
                match self.send(|tx| BismuthFSRequest::MkNode {
                    parent,
                    name: name_string.to_string(),
                    user_id: _req.uid(),
                    group_id: _req.gid(),
                    perm: mode & !umask,
                    is_dir: true,
                    response: tx,
                }) {
                    Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
                    Err(DBFSErr(errno, dberr)) => {
                        if let Some(error) = dberr {
                            error!(
                                "Database error trying to make INode({})/{name_string} dir: {error}",
                                parent.0.to_string()
                            );
                        }

                        reply.error(errno);
                    }
                }
            }
            _ => reply.error(Errno::EINVAL),
        }
    }

    fn unlink(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] unlink(parent: {parent:#x?}, name: {name:?})",);
        reply.error(Errno::ENOSYS);
    }

    fn rmdir(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] rmdir(parent: {parent:#x?}, name: {name:?})",);
        reply.error(Errno::ENOSYS);
    }

    fn symlink(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        link_name: &std::ffi::OsStr,
        target: &std::path::Path,
        reply: fuser::ReplyEntry,
    ) {
        warn!(
            "[Not Implemented] symlink(parent: {parent:#x?}, link_name: {link_name:?}, target: {target:?})",
        );
        reply.error(Errno::EPERM);
    }

    fn rename(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        newparent: fuser::INodeNo,
        newname: &std::ffi::OsStr,
        flags: fuser::RenameFlags,
        reply: fuser::ReplyEmpty,
    ) {
        warn!(
            "[Not Implemented] rename(parent: {parent:#x?}, name: {name:?}, \
            newparent: {newparent:#x?}, newname: {newname:?}, flags: {flags})",
        );
        reply.error(Errno::ENOSYS);
    }

    fn link(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        newparent: fuser::INodeNo,
        newname: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        warn!(
            "[Not Implemented] link(ino: {ino:#x?}, newparent: {newparent:#x?}, newname: {newname:?})"
        );
        reply.error(Errno::EPERM);
    }

    fn open(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _flags: fuser::OpenFlags,
        reply: fuser::ReplyOpen,
    ) {
        reply.opened(fuser::FileHandle(0), fuser::FopenFlags::empty());
    }

    fn read(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        size: u32,
        flags: fuser::OpenFlags,
        lock_owner: Option<fuser::LockOwner>,
        reply: fuser::ReplyData,
    ) {
        warn!(
            "[Not Implemented] read(ino: {ino:#x?}, fh: {fh}, offset: {offset}, \
            size: {size}, flags: {flags:#x?}, lock_owner: {lock_owner:?})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn write(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        data: &[u8],
        write_flags: fuser::WriteFlags,
        flags: fuser::OpenFlags,
        lock_owner: Option<fuser::LockOwner>,
        reply: fuser::ReplyWrite,
    ) {
        warn!(
            "[Not Implemented] write(ino: {ino:#x?}, fh: {fh}, offset: {offset}, \
            data.len(): {}, write_flags: {write_flags:#x?}, flags: {flags:#x?}, \
            lock_owner: {lock_owner:?})",
            data.len()
        );
        reply.error(Errno::ENOSYS);
    }

    fn flush(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        lock_owner: fuser::LockOwner,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] flush(ino: {ino:#x?}, fh: {fh}, lock_owner: {lock_owner:?})");
        reply.error(Errno::ENOSYS);
    }

    fn release(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _flags: fuser::OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok();
    }

    fn fsync(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] fsync(ino: {ino:#x?}, fh: {fh}, datasync: {datasync})");
        reply.error(Errno::ENOSYS);
    }

    fn opendir(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _flags: fuser::OpenFlags,
        reply: fuser::ReplyOpen,
    ) {
        reply.opened(fuser::FileHandle(0), fuser::FopenFlags::empty());
    }

    fn readdir(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        reply: fuser::ReplyDirectory,
    ) {
        warn!("[Not Implemented] readdir(ino: {ino:#x?}, fh: {fh}, offset: {offset})");
        reply.error(Errno::ENOSYS);
    }

    fn readdirplus(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        warn!("[Not Implemented] readdirplus(ino: {ino:#x?}, fh: {fh}, offset: {offset})");
        reply.error(Errno::ENOSYS);
    }

    fn releasedir(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _flags: fuser::OpenFlags,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok();
    }

    fn fsyncdir(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] fsyncdir(ino: {ino:#x?}, fh: {fh}, datasync: {datasync})");
        reply.error(Errno::ENOSYS);
    }

    fn statfs(&self, _req: &fuser::Request, _ino: fuser::INodeNo, reply: fuser::ReplyStatfs) {
        reply.statfs(0, 0, 0, 0, 0, 512, 255, 0);
    }

    fn setxattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        name: &std::ffi::OsStr,
        _value: &[u8],
        flags: i32,
        position: u32,
        reply: fuser::ReplyEmpty,
    ) {
        warn!(
            "[Not Implemented] setxattr(ino: {ino:#x?}, name: {name:?}, \
            flags: {flags:#x?}, position: {position})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn getxattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        name: &std::ffi::OsStr,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        warn!("[Not Implemented] getxattr(ino: {ino:#x?}, name: {name:?}, size: {size})");
        reply.error(Errno::ENOSYS);
    }

    fn listxattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        warn!("[Not Implemented] listxattr(ino: {ino:#x?}, size: {size})");
        reply.error(Errno::ENOSYS);
    }

    fn removexattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] removexattr(ino: {ino:#x?}, name: {name:?})");
        reply.error(Errno::ENOSYS);
    }

    fn access(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        mask: fuser::AccessFlags,
        reply: fuser::ReplyEmpty,
    ) {
        warn!("[Not Implemented] access(ino: {ino:#x?}, mask: {mask})");
        reply.error(Errno::ENOSYS);
    }

    fn create(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        warn!(
            "[Not Implemented] create(parent: {parent:#x?}, name: {name:?}, mode: {mode}, \
            umask: {umask:#x?}, flags: {flags:#x?})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn getlk(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        lock_owner: fuser::LockOwner,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        reply: fuser::ReplyLock,
    ) {
        warn!(
            "[Not Implemented] getlk(ino: {ino:#x?}, fh: {fh}, lock_owner: {lock_owner}, \
            start: {start}, end: {end}, typ: {typ}, pid: {pid})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn setlk(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        lock_owner: fuser::LockOwner,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        sleep: bool,
        reply: fuser::ReplyEmpty,
    ) {
        warn!(
            "[Not Implemented] setlk(ino: {ino:#x?}, fh: {fh}, lock_owner: {lock_owner}, \
            start: {start}, end: {end}, typ: {typ}, pid: {pid}, sleep: {sleep})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn bmap(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        blocksize: u32,
        idx: u64,
        reply: fuser::ReplyBmap,
    ) {
        warn!("[Not Implemented] bmap(ino: {ino:#x?}, blocksize: {blocksize}, idx: {idx})",);
        reply.error(Errno::ENOSYS);
    }

    fn ioctl(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        flags: fuser::IoctlFlags,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        reply: fuser::ReplyIoctl,
    ) {
        warn!(
            "[Not Implemented] ioctl(ino: {ino:#x?}, fh: {fh}, flags: {flags}, \
            cmd: {cmd}, in_data.len(): {}, out_size: {out_size})",
            in_data.len()
        );
        reply.error(Errno::ENOSYS);
    }

    fn poll(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        ph: fuser::PollNotifier,
        events: fuser::PollEvents,
        flags: fuser::PollFlags,
        reply: fuser::ReplyPoll,
    ) {
        warn!(
            "[Not Implemented] poll(ino: {ino:#x?}, fh: {fh}, \
            ph: {ph:?}, events: {events}, flags: {flags})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn fallocate(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        length: u64,
        mode: i32,
        reply: fuser::ReplyEmpty,
    ) {
        warn!(
            "[Not Implemented] fallocate(ino: {ino:#x?}, fh: {fh}, \
            offset: {offset}, length: {length}, mode: {mode})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn lseek(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: i64,
        whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        warn!(
            "[Not Implemented] lseek(ino: {ino:#x?}, fh: {fh}, \
            offset: {offset}, whence: {whence})"
        );
        reply.error(Errno::ENOSYS);
    }

    fn copy_file_range(
        &self,
        _req: &fuser::Request,
        ino_in: fuser::INodeNo,
        fh_in: fuser::FileHandle,
        offset_in: u64,
        ino_out: fuser::INodeNo,
        fh_out: fuser::FileHandle,
        offset_out: u64,
        len: u64,
        flags: fuser::CopyFileRangeFlags,
        reply: fuser::ReplyWrite,
    ) {
        warn!(
            "[Not Implemented] copy_file_range(ino_in: {ino_in:#x?}, fh_in: {fh_in}, \
            offset_in: {offset_in}, ino_out: {ino_out:#x?}, fh_out: {fh_out}, \
            offset_out: {offset_out}, len: {len}, flags: {flags:?})"
        );
        reply.error(Errno::ENOSYS);
    }
}
