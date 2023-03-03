/// Monoio Driver.
pub(crate) mod op;
pub(crate) mod shared_fd;
#[cfg(feature = "sync")]
pub(crate) mod thread;

mod uring;

mod util;

use std::{
    io,
    task::{Context, Poll},
    time::Duration,
};

pub use self::uring::IoUringDriver;
use self::{
    op::{CompletionMeta, Op, OpAble},
    uring::UringInner,
};

/// Unpark a runtime of another thread.
pub(crate) mod unpark {
    #[allow(unreachable_pub)]
    pub trait Unpark: Sync + Send + 'static {
        /// Unblocks a thread that is blocked by the associated `Park` handle.
        ///
        /// Calling `unpark` atomically makes available the unpark token, if it
        /// is not already available.
        ///
        /// # Panics
        ///
        /// This function **should** not panic, but ultimately, panics are left
        /// as an implementation detail. Refer to the documentation for
        /// the specific `Unpark` implementation
        fn unpark(&self) -> std::io::Result<()>;
    }
}

impl unpark::Unpark for Box<dyn unpark::Unpark> {
    fn unpark(&self) -> io::Result<()> {
        (**self).unpark()
    }
}

impl unpark::Unpark for std::sync::Arc<dyn unpark::Unpark> {
    fn unpark(&self) -> io::Result<()> {
        (**self).unpark()
    }
}

/// Core driver trait.
pub trait Driver {
    /// Run with driver TLS.
    fn with<R>(&self, f: impl FnOnce() -> R) -> R;
    /// Submit ops to kernel and process returned events.
    fn submit(&self) -> io::Result<()>;
    /// Wait infinitely and process returned events.
    fn park(&self) -> io::Result<()>;
    /// Wait with timeout and process returned events.
    fn park_timeout(&self, duration: Duration) -> io::Result<()>;

    /// The struct to wake thread from another.
    #[cfg(feature = "sync")]
    type Unpark: unpark::Unpark;

    /// Get Unpark.
    #[cfg(feature = "sync")]
    fn unpark(&self) -> Self::Unpark;
}

scoped_thread_local!(pub(crate) static CURRENT: Inner);

pub struct Inner(std::rc::Rc<std::cell::UnsafeCell<UringInner>>);

impl Inner {
    fn submit_with<T: OpAble>(&self, data: T) -> io::Result<Op<T>> {
        UringInner::submit_with_data(&self.0, data)
    }

    #[allow(unused)]
    fn poll_op<T: OpAble>(
        &self,
        data: &mut T,
        index: usize,
        cx: &mut Context<'_>,
    ) -> Poll<CompletionMeta> {
        UringInner::poll_op(&self.0, index, cx)
    }

    #[allow(unused)]
    fn drop_op<T: 'static>(&self, index: usize, data: &mut Option<T>) {
        UringInner::drop_op(&self.0, index, data)
    }

    #[allow(unused)]
    pub(super) unsafe fn cancel_op(&self, op_canceller: &op::OpCanceller) {
        UringInner::cancel_op(&self.0, op_canceller.index)
    }
}

/// The unified UnparkHandle.
#[cfg(feature = "sync")]
pub use crate::driver::uring::UnparkHandle;
