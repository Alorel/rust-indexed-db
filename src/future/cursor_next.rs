use crate::cursor::BaseCursor;
use crate::primitive::{TryFromJs, TryFromJsExt};
use fancy_constructor::new;
use internal_macros::FutureFromPollUnpinned;
use sealed::sealed;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

/// Future for resolving cursors' `next` & `next_key` method calls.
#[derive(FutureFromPollUnpinned, new)]
#[new(vis())]
pub struct CursorNextRequest<'a, T> {
    cursor: &'a mut BaseCursor,
    read_sys: fn(&BaseCursor) -> crate::Result<JsValue>,
    fmt_sys: fn(JsValue) -> crate::Result<T>,
}

impl<'a, T> CursorNextRequest<'a, T>
where
    T: TryFromJs,
{
    #[inline]
    pub(crate) fn key_js(cursor: &'a mut BaseCursor) -> Self {
        Self::new(cursor, BaseCursor::key_sys, T::from_js_base)
    }

    #[inline]
    pub(crate) fn value_js(cursor: &'a mut BaseCursor) -> Self {
        Self::new(cursor, BaseCursor::value_sys, T::from_js_base)
    }
}

#[cfg(feature = "serde")]
impl<'a, T> CursorNextRequest<'a, T>
where
    T: crate::serde::DeserialiseFromJs,
{
    #[inline]
    pub(crate) fn key_serde(cursor: &'a mut BaseCursor) -> Self {
        Self::new(
            cursor,
            BaseCursor::key_sys,
            crate::serde::DeserialiseFromJs::deserialise_from_js,
        )
    }

    #[inline]
    pub(crate) fn value_serde(cursor: &'a mut BaseCursor) -> Self {
        Self::new(
            cursor,
            BaseCursor::value_sys,
            crate::serde::DeserialiseFromJs::deserialise_from_js,
        )
    }
}

#[sealed]
impl<T> super::PollUnpinned for CursorNextRequest<'_, T> {
    type Output = crate::Result<Option<T>>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        let Self {
            read_sys, fmt_sys, ..
        } = *self;

        self.cursor
            .poll_state(cx, move |cur| read_sys(cur).and_then(fmt_sys))
    }
}
