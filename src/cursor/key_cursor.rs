use super::{BaseCursor, CursorSys};
use crate::future::CursorNextRequest;
use crate::internal_utils::SystemRepr;
use crate::primitive::TryFromJs;
use accessory::Accessors;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use internal_macros::errdoc;

/// An [`IDBCursor`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor) implementation.
#[derive(Debug, Accessors, Deref, DerefMut, AsRef, AsMut)]
pub struct KeyCursor<'a, Qs> {
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    base: BaseCursor,

    /// The thing that spawned this cursor.
    #[access(get(cp))]
    source: &'a Qs,
}

impl<'a, Qs> KeyCursor<'a, Qs> {
    pub(crate) fn new(base: CursorSys, source: &'a Qs) -> Self {
        Self {
            base: BaseCursor::new(base),
            source,
        }
    }

    /// Get the next key in the cursor.
    ///
    /// Equivalent to calling [`continue()`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/continue)
    /// followed by [`key`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/key) in JS.
    #[inline]
    #[errdoc(Cursor(TransactionInactiveError, InvalidStateError))]
    pub fn next_key<T>(&mut self) -> CursorNextRequest<'_, T>
    where
        T: TryFromJs,
    {
        CursorNextRequest::key_js(self)
    }

    /// Mirror of [`Self::next_key`] for `serde`-deserialisable keys.
    #[inline]
    #[cfg(feature = "serde")]
    pub fn next_key_ser<T>(&mut self) -> CursorNextRequest<'_, T>
    where
        T: crate::serde::DeserialiseFromJs,
    {
        CursorNextRequest::key_serde(self)
    }

    /// Convert this cursor into a stream of primitive keys.
    #[cfg(feature = "streams")]
    #[inline]
    #[must_use]
    pub fn key_stream<T>(self) -> super::Stream<Self, T>
    where
        T: TryFromJs,
    {
        super::Stream::new_js_key(self)
    }

    /// Convert this cursor into a stream of `serde`-deserialisable keys.
    #[cfg(all(feature = "streams", feature = "serde"))]
    #[inline]
    #[must_use]
    pub fn key_stream_ser<T>(self) -> super::Stream<Self, T>
    where
        T: crate::serde::DeserialiseFromJs,
    {
        super::Stream::new_ser_key(self)
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl<Qs> crate::internal_utils::SystemRepr for KeyCursor<'_, Qs> {
    type Repr = <BaseCursor as SystemRepr>::Repr;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        self.base.as_sys()
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.base.into_sys()
    }
}

#[cfg(feature = "indices")]
const _: () = {
    use crate::primitive::TryToJs;
    use wasm_bindgen::prelude::*;

    impl KeyCursor<'_, crate::index::Index<'_>> {
        /// Advance the cursor to the record whose key matches the `key` as well as whose primary key matches the
        /// `primary_key`.
        ///
        /// A typical use case, is to resume the iteration where a previous cursor has been closed, without having to
        /// compare the keys one by one.
        ///
        /// Equivalent to calling
        /// [`continuePrimaryKey(key, primary_key)`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/continuePrimaryKey)
        /// in JS.
        #[errdoc(Cursor(
            TransactionInactiveError,
            DataError,
            InvalidStateError,
            InvalidAccessError
        ))]
        #[allow(clippy::missing_errors_doc)]
        pub async fn continue_to_primary_key<K, PK>(
            &mut self,
            key: K,
            primary_key: PK,
        ) -> crate::Result<()>
        where
            K: TryToJs,
            PK: TryToJs,
        {
            let key = key.try_to_js()?;
            let primary_key = primary_key.try_to_js()?;
            self.continue_to_primary_key_common(&key, &primary_key)
                .await
        }

        /// [`Self::continue_to_primary_key`] mirror using `serde`.
        #[cfg(feature = "serde")]
        #[allow(clippy::missing_errors_doc)]
        pub async fn continue_to_primary_key_ser<K, PK>(
            &mut self,
            key: K,
            primary_key: PK,
        ) -> crate::Result<()>
        where
            K: crate::serde::SerialiseToJs,
            PK: crate::serde::SerialiseToJs,
        {
            let key = key.serialise_to_js()?;
            let primary_key = primary_key.serialise_to_js()?;
            self.continue_to_primary_key_common(&key, &primary_key)
                .await
        }

        async fn continue_to_primary_key_common(
            &mut self,
            key: &JsValue,
            value: &JsValue,
        ) -> crate::Result<()> {
            self.as_sys().continue_primary_key(key, value)?;
            self.req().await?;
            self.on_cursor_position_reset();
            Ok(())
        }
    }
};
