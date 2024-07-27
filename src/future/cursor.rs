use super::Request;
use crate::cursor::{Cursor, CursorSys, KeyCursor};
use internal_macros::FutureFromPollUnpinned;
use sealed::sealed;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use wasm_bindgen::JsCast;

/// A Future for opening a cursor. Resolves to `None` if the cursor is empty.
#[derive(FutureFromPollUnpinned)]
pub struct CursorRequest<'a, T, Qs> {
    source: &'a Qs,
    req: Request,
    cursor_ty: PhantomData<T>,
}

impl<'a, T, Qs> CursorRequest<'a, T, Qs> {
    pub(crate) fn new(req: web_sys::IdbRequest, source: &'a Qs) -> Self {
        Self {
            source,
            req: Request::new(req),
            cursor_ty: PhantomData,
        }
    }
}

#[sealed]
impl<'a, T, Qs> super::PollUnpinned for CursorRequest<'a, T, Qs>
where
    T: CursorKind<'a, Qs>,
{
    type Output = crate::Result<Option<T>>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.req.poll_unpinned(cx) {
            Poll::Ready(res) => Poll::Ready(match res {
                Ok(js) => Ok(match js.dyn_into::<web_sys::IdbCursor>() {
                    Ok(sys) => Some(T::construct(sys.unchecked_into(), self.source)),
                    Err(_) => None,
                }),
                Err(e) => Err(e),
            }),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[sealed]
#[allow(missing_docs)]
pub trait CursorKind<'a, Qs> {
    #[doc(hidden)]
    fn construct(sys: CursorSys, query_source: &'a Qs) -> Self;
}

#[sealed]
impl<'a, Qs> CursorKind<'a, Qs> for Cursor<'a, Qs> {
    #[inline]
    fn construct(sys: CursorSys, query_source: &'a Qs) -> Self {
        Cursor::new(sys, query_source)
    }
}

#[sealed]
impl<'a, Qs> CursorKind<'a, Qs> for KeyCursor<'a, Qs> {
    #[inline]
    fn construct(sys: CursorSys, query_source: &'a Qs) -> Self {
        KeyCursor::new(sys, query_source)
    }
}
