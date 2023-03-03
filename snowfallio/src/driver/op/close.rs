use std::{io, os::unix::io::RawFd};

use io_uring::{opcode, types};

use super::{Op, OpAble};

pub(crate) struct Close {
    fd: RawFd,
}

impl Op<Close> {
    #[allow(unused)]
    pub(crate) fn close(fd: RawFd) -> io::Result<Op<Close>> {
        Op::try_submit_with(Close { fd })
    }
}

impl OpAble for Close {
    fn uring_op(&mut self) -> io_uring::squeue::Entry {
        opcode::Close::new(types::Fd(self.fd)).build()
    }
}
