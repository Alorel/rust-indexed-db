//! [`IDBCursor`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor) &
//! [`IDBCursorWithValue`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursorWithValue) implementations.

pub use base_cursor::BaseCursor;
pub(crate) use cursor_sys::CursorSys;
pub use key_cursor::KeyCursor;
pub use update::Update;
pub use web_sys::IdbCursorDirection as CursorDirection;

use crate::future::{CursorNextRequest, VoidRequest};
use crate::internal_utils::SystemRepr;
use crate::primitive::TryFromJs;
use internal_macros::errdoc;

mod base_cursor;
pub(crate) mod cursor_sys;
mod key_cursor;
mod update;

iffeat! {
    #[cfg(feature = "streams")]
    mod stream;
    pub use stream::*;
}

/// An [`IDBCursorWithValue`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursorWithValue)
/// implementation.
#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct Cursor<'a, Qs>(KeyCursor<'a, Qs>);

impl<'a, Qs> Cursor<'a, Qs> {
    pub(crate) fn new(base: CursorSys, source: &'a Qs) -> Self {
        Self(KeyCursor::new(base, source))
    }

    /// Get the key associated with the record at the cursor's current position.
    ///
    /// Refers to the key path or generated key when called on an
    /// [`ObjectStore`](crate::object_store::ObjectStore) and the indexed key path when called on
    /// an [`Index`](crate::index::Index).
    ///
    /// Returns `None` if the cursor's iterated past its end.
    #[allow(clippy::missing_errors_doc)]
    pub fn key<T>(&self) -> crate::Result<Option<T>>
    where
        Option<T>: TryFromJs,
    {
        TryFromJs::from_js(self.as_sys().key()?).map_err(Into::into)
    }

    /// [`Self::key`] mirror for `serde`-deserialisable keys.
    #[cfg(feature = "serde")]
    #[allow(clippy::missing_errors_doc)]
    pub fn key_ser<T>(&self) -> crate::Result<Option<T>>
    where
        Option<T>: crate::serde::DeserialiseFromJs,
    {
        crate::serde::DeserialiseFromJs::deserialise_from_js(self.as_sys().key()?)
    }

    /// Get the next record in the cursor.
    ///
    /// Equivalent to calling [`continue()`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/continue)
    /// followed by [`value`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursorWithValue/value) in JS.
    #[errdoc(Cursor(TransactionInactiveError, InvalidStateError))]
    #[inline]
    pub fn next_record<T>(&mut self) -> CursorNextRequest<'_, T>
    where
        T: TryFromJs,
    {
        CursorNextRequest::value_js(self)
    }

    /// Mirror of [`Self::next_record`] for `serde`-deserialisable values.
    #[cfg(feature = "serde")]
    pub fn next_record_ser<T>(&mut self) -> CursorNextRequest<'_, T>
    where
        T: crate::serde::DeserialiseFromJs,
    {
        CursorNextRequest::value_serde(self)
    }

    /// Delete the record at the cursor's current position. [`key`](Self::key) and
    /// [`primary_key`](BaseCursor::primary_key) will continue to refer to the current cursor position until the
    /// cursor is advanced.
    #[errdoc(Cursor(TransactionInactiveError, ReadOnlyError, InvalidStateError))]
    #[allow(clippy::missing_errors_doc)]
    pub fn delete(&mut self) -> crate::Result<VoidRequest> {
        let req = self.as_sys().delete()?;
        self.invalidate_current();
        Ok(VoidRequest::new(req))
    }

    /// Overwrite the value at the given position with the given value. If the cursor points to a record that has just
    /// been deleted, a new record is created.
    ///
    /// The value should implement [`TryToJs`](crate::primitive::TryToJs) or, if the `serde` feature is enabled,
    /// [`Serialize`](serde::Serialize).
    ///
    /// If a key type is specified in the builder, the return value will contain the key of the updated record.
    #[errdoc(Cursor(
        TransactionInactiveError,
        ReadOnlyError,
        InvalidStateError,
        DataErrorUpdate,
        DataCloneError,
    ))]
    #[inline]
    pub fn update<V>(&self, value: V) -> Update<'_, V> {
        Update::new(self, value)
    }

    /// Convert this cursor into a stream of primitive keys.
    #[cfg(feature = "streams")]
    #[inline]
    #[must_use]
    pub fn key_stream<T>(self) -> Stream<KeyCursor<'a, Qs>, T>
    where
        T: TryFromJs,
    {
        self.0.key_stream()
    }

    /// Convert this cursor into a stream of `serde`-deserialisable keys.
    #[cfg(all(feature = "streams", feature = "serde"))]
    #[inline]
    #[must_use]
    pub fn key_stream_ser<T>(self) -> Stream<KeyCursor<'a, Qs>, T>
    where
        T: crate::serde::DeserialiseFromJs,
    {
        self.0.key_stream_ser()
    }

    /// Convert this cursor into a stream of primitive values.
    #[cfg(feature = "streams")]
    #[must_use]
    pub fn stream<T>(self) -> Stream<Self, T>
    where
        T: TryFromJs,
    {
        Stream::new_js(self)
    }

    /// Convert this cursor into a stream of `serde`-deserialisable values.
    #[cfg(all(feature = "streams", feature = "serde"))]
    #[must_use]
    pub fn stream_ser<T>(self) -> Stream<Self, T>
    where
        T: serde::de::DeserializeOwned,
    {
        Stream::new_ser(self)
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl<'a, Qs> crate::internal_utils::SystemRepr for Cursor<'a, Qs> {
    type Repr = <KeyCursor<'a, Qs> as SystemRepr>::Repr;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.0.as_sys()
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.0.into_sys()
    }
}

impl<Qs> AsMut<BaseCursor> for Cursor<'_, Qs> {
    #[inline]
    fn as_mut(&mut self) -> &mut BaseCursor {
        self.0.as_mut()
    }
}

impl<Qs> AsRef<BaseCursor> for Cursor<'_, Qs> {
    #[inline]
    fn as_ref(&self) -> &BaseCursor {
        self.0.as_ref()
    }
}
