use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::DomException;

use super::super::IdbRequestFuture;
use crate::idb_cursor::IdbCursor;
use crate::idb_query_source::IdbQuerySource;
use crate::internal_utils::NightlyUnwrap;
use crate::request::IdbRequestRef;

/// A [`Future`] that resolves to an [`IdbCursor`]
#[derive(Debug)]
pub struct IdbCursorFuture<'a, T: IdbQuerySource> {
    inner: IdbRequestFuture,
    source: &'a T,
    req: Rc<IdbRequestRef>,
}

impl<'a, T: IdbQuerySource> IdbCursorFuture<'a, T> {
    pub(crate) fn new(
        req: Result<web_sys::IdbRequest, JsValue>,
        source: &'a T,
    ) -> Result<Self, DomException> {
        let request = Rc::new(IdbRequestRef::new(req?));
        let inner = IdbRequestFuture::new_with_rc(request, true);
        let req = inner.strong_request();

        Ok(Self { inner, source, req })
    }

    /// Actual future poll method
    pub(crate) fn do_poll(
        &self,
        ctx: &Context<'_>,
    ) -> Poll<Result<Option<IdbCursor<'a, T>>, DomException>> {
        self.inner.do_poll(ctx).map(|res| self.on_ready(res))
    }

    fn on_ready(
        &self,
        res: Result<Option<JsValue>, DomException>,
    ) -> Result<Option<IdbCursor<'a, T>>, DomException> {
        let raw = res?.nightly_unwrap();
        let opt = if raw.is_null() {
            None
        } else {
            let cur = IdbCursor::new(raw.unchecked_into(), self.source, Rc::clone(&self.req));
            Some(cur)
        };
        Ok(opt)
    }
}

impl<'a, T: IdbQuerySource> Future for IdbCursorFuture<'a, T> {
    type Output = Result<Option<IdbCursor<'a, T>>, DomException>;

    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(ctx)
    }
}
