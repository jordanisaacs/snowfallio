use std::{
    cell::UnsafeCell,
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
    rc::Rc,
};

// Tracks in-flight operations on a file descriptor. Ensures all in-flight
// operations complete before submitting the close.
#[derive(Clone, Debug)]
pub(crate) struct SharedFd {
    inner: Rc<Inner>,
}

struct Inner {
    // Open file descriptor
    fd: RawFd,

    // Waker to notify when the close operation completes.
    state: UnsafeCell<UringState>,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner").field("fd", &self.fd).finish()
    }
}

enum UringState {
    /// Initial state
    Init,

    /// Waiting for all in-flight operation to complete.
    Waiting(Option<std::task::Waker>),

    /// The FD is closing
    Closing(super::op::Op<super::op::close::Close>),

    /// The FD is fully closed
    Closed,
}

impl AsRawFd for SharedFd {
    fn as_raw_fd(&self) -> RawFd {
        self.raw_fd()
    }
}

impl SharedFd {
    #[allow(unreachable_code, unused)]
    pub(crate) fn new(fd: RawFd) -> SharedFd {
        let state = UringState::Init;

        SharedFd {
            inner: Rc::new(Inner {
                fd,
                state: UnsafeCell::new(state),
            }),
        }
    }

    /// Returns the RawFd
    pub(crate) fn raw_fd(&self) -> RawFd {
        self.inner.fd
    }

    /// Try unwrap Rc, then deregister if registered and return rawfd.
    /// Note: this action will consume self and return rawfd without closing it.
    pub(crate) fn try_unwrap(self) -> Result<RawFd, Self> {
        let fd = self.inner.fd;
        match Rc::try_unwrap(self.inner) {
            Ok(_inner) => Ok(fd),
            Err(inner) => Err(Self { inner }),
        }
    }

    /// An FD cannot be closed until all in-flight operation have completed.
    /// This prevents bugs where in-flight reads could operate on the incorrect
    /// file descriptor.
    pub(crate) async fn close(self) {
        // Here we only submit close op for uring mode.
        // Fd will be closed when Inner drops for legacy mode.
        let fd = self.inner.fd;
        let mut this = self;
        let uring_state = unsafe { &mut *this.inner.state.get() };
        if Rc::get_mut(&mut this.inner).is_some() {
            *uring_state = match super::op::Op::close(fd) {
                Ok(op) => UringState::Closing(op),
                Err(_) => {
                    let _ = unsafe { std::fs::File::from_raw_fd(fd) };
                    return;
                }
            };
        }
        this.inner.closed().await;
    }
}

impl Inner {
    /// Completes when the FD has been closed.
    /// Should only be called for uring mode.
    async fn closed(&self) {
        use std::{future::Future, pin::Pin, task::Poll};

        crate::macros::support::poll_fn(|cx| {
            let uring_state = unsafe { &mut *self.state.get() };

            match uring_state {
                UringState::Init => {
                    *uring_state = UringState::Waiting(Some(cx.waker().clone()));
                    Poll::Pending
                }
                UringState::Waiting(Some(waker)) => {
                    if !waker.will_wake(cx.waker()) {
                        *waker = cx.waker().clone();
                    }

                    Poll::Pending
                }
                UringState::Waiting(None) => {
                    *uring_state = UringState::Waiting(Some(cx.waker().clone()));
                    Poll::Pending
                }
                UringState::Closing(op) => {
                    // Nothing to do if the close operation failed.
                    let _ = ready!(Pin::new(op).poll(cx));
                    *uring_state = UringState::Closed;
                    Poll::Ready(())
                }
                UringState::Closed => Poll::Ready(()),
            }
        })
        .await;
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let fd = self.fd;
        let state = unsafe { &mut *self.state.get() };
        #[allow(unreachable_patterns)]
        match state {
            UringState::Init | UringState::Waiting(..) => {
                if super::op::Op::close(fd).is_err() {
                    let _ = unsafe { std::fs::File::from_raw_fd(fd) };
                };
            }
            _ => {}
        }
    }
}
