use super::{BaseCursor, Cursor, KeyCursor};
use crate::future::VoidRequest;
use crate::primitive::{TryFromJs, TryFromJsExt};
use derive_more::{Deref, DerefMut};
use internal_macros::errdoc;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

/// A stream off a [`Cursor`] or [`KeyCursor`].
#[derive(Deref, DerefMut)]
pub struct Stream<Cur, T> {
    #[deref]
    #[deref_mut]
    cursor: Cur,
    read_sys: fn(&BaseCursor) -> crate::Result<JsValue>,
    fmt_sys: fn(JsValue) -> crate::Result<T>,
}

impl<'a, Qs, T> Stream<KeyCursor<'a, Qs>, T> {
    #[inline]
    pub(super) fn new_js_key(cursor: KeyCursor<'a, Qs>) -> Self
    where
        T: TryFromJs,
    {
        Self {
            cursor,
            read_sys: BaseCursor::key_sys,
            fmt_sys: T::from_js_base,
        }
    }

    #[cfg(feature = "serde")]
    #[inline]
    pub(super) fn new_ser_key(cursor: KeyCursor<'a, Qs>) -> Self
    where
        T: crate::serde::DeserialiseFromJs,
    {
        Self {
            cursor,
            read_sys: BaseCursor::key_sys,
            fmt_sys: T::deserialise_from_js,
        }
    }
}

impl<'a, Qs, T> Stream<Cursor<'a, Qs>, T> {
    #[inline]
    pub(super) fn new_js(cursor: Cursor<'a, Qs>) -> Self
    where
        T: TryFromJs,
    {
        Self {
            cursor,
            read_sys: BaseCursor::value_sys,
            fmt_sys: T::from_js_base,
        }
    }

    #[cfg(feature = "serde")]
    #[inline]
    pub(super) fn new_ser(cursor: Cursor<'a, Qs>) -> Self
    where
        T: crate::serde::DeserialiseFromJs,
    {
        Self {
            cursor,
            read_sys: BaseCursor::value_sys,
            fmt_sys: T::deserialise_from_js,
        }
    }

    /// Delete the record at the cursor's current position. Inapplicable to key streams.
    ///
    /// See [`Cursor::delete`](Cursor::delete) for more information.
    #[inline]
    #[errdoc(Cursor(TransactionInactiveError, ReadOnlyError, InvalidStateError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn delete(&mut self) -> crate::Result<VoidRequest> {
        self.cursor.delete()
    }

    /// Overwrite the value at the given position with the given value. Inapplicable to key streams.
    ///
    /// See [`Cursor::update`](Cursor::update) for more information.
    #[errdoc(Cursor(
        TransactionInactiveError,
        ReadOnlyError,
        InvalidStateError,
        DataErrorUpdate,
        DataCloneError
    ))]
    pub fn update<V>(&self, value: V) -> super::Update<'_, V> {
        self.cursor.update(value)
    }

    /// Get the key associated with the record at the cursor's current position. Inapplicable to key streams.
    ///
    /// Returns `None` if the stream has been iterated past its end.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub fn key<K>(&self) -> crate::Result<Option<K>>
    where
        Option<K>: TryFromJs,
    {
        self.cursor.key()
    }

    /// Get the `serde`-serialisable key associated with the record at the cursor's current position. Inapplicable to
    /// key streams.
    #[cfg(feature = "serde")]
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub fn key_ser<K>(&self) -> crate::Result<Option<K>>
    where
        Option<K>: crate::serde::DeserialiseFromJs,
    {
        self.cursor.key_ser()
    }
}

impl<Cur, T> futures_core::Stream for Stream<Cur, T>
where
    Cur: AsMut<BaseCursor> + Unpin,
{
    type Item = crate::Result<T>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            ref mut cursor,
            read_sys,
            fmt_sys,
        } = *self;

        let cur_poll = cursor
            .as_mut()
            .poll_state(cx, move |cursor| read_sys(cursor).and_then(fmt_sys));

        match cur_poll {
            Poll::Ready(Ok(Some(v))) => Poll::Ready(Some(Ok(v))),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(None)) => Poll::Ready(None),
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
        }
    }
}

impl<Cur, T> futures_core::FusedStream for Stream<Cur, T>
where
    Cur: AsMut<BaseCursor> + AsRef<BaseCursor> + Unpin,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.cursor.as_ref().is_finished()
    }
}
