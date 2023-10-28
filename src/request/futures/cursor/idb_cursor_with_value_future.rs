use fancy_constructor::new;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use web_sys::DomException;

use crate::idb_cursor::{IdbCursor, IdbCursorWithValue};
use crate::idb_query_source::IdbQuerySource;

use super::IdbCursorFuture;

/// A [`Future`] that resolves to an [`IdbCursorWithValue`]
#[derive(Debug, new)]
#[new(vis(pub(crate)))]
pub struct IdbCursorWithValueFuture<'a, T: IdbQuerySource>(IdbCursorFuture<'a, T>);

impl<'a, T: IdbQuerySource> IdbCursorWithValueFuture<'a, T> {
    fn on_ready(
        res: Result<Option<IdbCursor<'a, T>>, DomException>,
    ) -> Result<Option<IdbCursorWithValue<'a, T>>, DomException> {
        Ok(res?.map(IdbCursorWithValue::new))
    }
}

impl<'a, T: IdbQuerySource> Future for IdbCursorWithValueFuture<'a, T> {
    type Output = Result<Option<IdbCursorWithValue<'a, T>>, DomException>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.do_poll(ctx).map(Self::on_ready)
    }
}
