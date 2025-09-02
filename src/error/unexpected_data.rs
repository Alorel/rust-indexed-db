use std::sync::PoisonError;

/// We reached a state that shouldn't have been allowed by the API.
#[derive(Debug, PartialEq, Eq, Copy, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum UnexpectedDataError {
    /// [`mpsc`](tokio::sync::mpsc) channel dropped unexpectedly.
    #[error("`mpsc` channel dropped.")]
    ChannelDropped,

    /// Expected there to be an error, but there wasn't one.
    #[error("Expected error not found.")]
    NoErrorFound,

    /// Expected there to be an
    /// [`EventTarget`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget), but there
    /// wasn't one.
    #[error("No event target.")]
    NoEventTarget,

    /// A [`Future`](std::future::Future) was polled in an unexpected way.
    #[error("`Future` polled unexpectedly.")]
    PollState,

    /// Expected a Transaction to exist, but it was not found.
    #[error("Expected the Transaction to exist, but it was not found.")]
    TransactionNotFound,

    /// Expected a Transaction to be aborted, but it was committed.
    #[error("Expected the Transaction to be aborted, but it was committed.")]
    TransactionCommitted,

    /// Expected a Transaction to be committed, but it was aborted.
    #[error("Expected the Transaction to be committed, but it was aborted.")]
    TransactionAborted,

    /// A mutex was poisoned.
    #[error("Mutex poisoned.")]
    PoisonedLock,
}

impl<T> From<PoisonError<T>> for UnexpectedDataError {
    #[inline]
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonedLock
    }
}
