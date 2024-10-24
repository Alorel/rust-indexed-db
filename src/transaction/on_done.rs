use super::TransactionSys;
use internal_macros::FutureFromPollUnpinned;
use sealed::sealed;
use std::task::{Context, Poll};
use wasm_evt_listener::Listener;

const EVT_ABORT: &str = "abort";
const EVT_ERROR: &str = "error";
const EVT_COMPLETE: &str = "complete";

/// How a transaction finished.
///
/// Note that this enum does not include the actual error in its [`Err`](TransactionFinishKind::Err) variant - the
/// error will be propagated to and should be handled by the piece of code that spawned the transaction.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum TransactionFinishKind {
    /// Finished with a `complete` event
    Ok,

    /// Finished with an `error` event
    Err,

    /// Finished with an `abort` event
    Abort,
}

/// A Future that resolves when a transaction finishes, successfully or not.
#[derive(FutureFromPollUnpinned)]
pub struct TransactionDone {
    tx: TransactionSys,
    on_abort: Listener,
    on_error: Listener,
    on_complete: Listener,
}

impl TransactionDone {
    pub(super) fn new(tx: TransactionSys) -> crate::Result<Self> {
        let on_abort = Listener::builder().build()?;
        let on_error = Listener::builder().build()?;
        let on_complete = Listener::builder().build()?;

        on_abort.add_to(EVT_ABORT, &tx)?;
        on_error.add_to(EVT_ERROR, &tx)?;
        on_complete.add_to(EVT_COMPLETE, &tx)?;

        Ok(Self {
            tx,
            on_abort,
            on_error,
            on_complete,
        })
    }

    fn finish_with(&mut self, kind: TransactionFinishKind) -> TransactionFinishKind {
        self.on_complete.close();
        self.on_abort.close();
        self.on_error.close();
        kind
    }
}

#[sealed]
impl crate::future::PollUnpinned for TransactionDone {
    type Output = TransactionFinishKind;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.on_complete.poll_recv(cx) {
            Poll::Ready(_) => Poll::Ready(self.finish_with(TransactionFinishKind::Ok)),
            Poll::Pending => match self.on_abort.poll_recv(cx) {
                Poll::Ready(_) => Poll::Ready(self.finish_with(TransactionFinishKind::Abort)),
                Poll::Pending => match self.on_error.poll_recv(cx) {
                    Poll::Ready(_) => Poll::Ready(self.finish_with(TransactionFinishKind::Err)),
                    Poll::Pending => Poll::Pending,
                },
            },
        }
    }
}

impl Drop for TransactionDone {
    fn drop(&mut self) {
        let _ = self.on_abort.rm_from(EVT_ABORT, &self.tx);
        let _ = self.on_error.rm_from(EVT_ERROR, &self.tx);
        let _ = self.on_complete.rm_from(EVT_COMPLETE, &self.tx);
    }
}

impl TransactionFinishKind {
    /// Returns `true` if the transaction finished successfully.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Convert the enum into a [`Result<()>`](crate::Result).
    #[allow(clippy::missing_errors_doc)]
    pub fn into_result(self) -> crate::Result<()> {
        let msg = match self {
            Self::Ok => return Ok(()),
            Self::Err => "The transaction errored",
            Self::Abort => "The transaction was aborted",
        };
        Err(js_sys::Error::new(msg).into())
    }
}

impl From<TransactionFinishKind> for crate::Result<()> {
    #[inline]
    fn from(finish: TransactionFinishKind) -> Self {
        finish.into_result()
    }
}

#[cfg(feature = "streams")]
impl futures_core::FusedFuture for TransactionDone {
    fn is_terminated(&self) -> bool {
        futures_core::FusedStream::is_terminated(&self.on_complete)
    }
}
