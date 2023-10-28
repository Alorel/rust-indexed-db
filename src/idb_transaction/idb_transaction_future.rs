use super::{IdbTransaction, IdbTransactionResult};
use fancy_constructor::new;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future that resolves once the transaction completes
#[derive(new)]
#[new(vis(pub(crate)))]
pub struct IdbTransactionFuture<'a> {
    tx: IdbTransaction<'a>,
}

impl<'a> Future for IdbTransactionFuture<'a> {
    type Output = IdbTransactionResult;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref v) = *self.tx.listeners.result.borrow() {
            Poll::Ready(v.clone())
        } else {
            self.tx
                .listeners
                .waker
                .borrow_mut()
                .replace(ctx.waker().clone());
            Poll::Pending
        }
    }
}
