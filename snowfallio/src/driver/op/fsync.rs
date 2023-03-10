use std::io;

use io_uring::{opcode, types};

use super::{super::shared_fd::SharedFd, Op, OpAble};

pub(crate) struct Fsync {
    #[allow(unused)]
    fd: SharedFd,
    data_sync: bool,
}

impl Op<Fsync> {
    pub(crate) fn fsync(fd: &SharedFd) -> io::Result<Op<Fsync>> {
        Op::submit_with(Fsync {
            fd: fd.clone(),
            data_sync: false,
        })
    }

    pub(crate) fn datasync(fd: &SharedFd) -> io::Result<Op<Fsync>> {
        Op::submit_with(Fsync {
            fd: fd.clone(),
            data_sync: true,
        })
    }
}

impl OpAble for Fsync {
    fn uring_op(&mut self) -> io_uring::squeue::Entry {
        let mut opc = opcode::Fsync::new(types::Fd(self.fd.raw_fd()));
        if self.data_sync {
            opc = opc.flags(types::FsyncFlags::DATASYNC)
        }
        opc.build()
    }
}
