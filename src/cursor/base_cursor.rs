use super::{CursorDirection, CursorSys};
use crate::future::{PollUnpinned, VoidRequest};
use crate::internal_utils::SystemRepr;
use crate::primitive::{TryFromJs, TryToJs};
use fancy_constructor::new;
use internal_macros::errdoc;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

/// Base implementation for all cursors.
#[derive(Debug, new)]
#[new(vis(pub(super)))]
pub struct BaseCursor {
    sys: CursorSys,

    #[new(default)]
    state: CursorState,
}

#[derive(Debug, Default)]
pub(crate) enum CursorState {
    /// We should try reading the current cursor position.
    #[default]
    ReadCurrent,

    /// We should query the next record.
    TryNext,

    /// We are currently reading the next record.
    ReadingNext(VoidRequest),
}

impl BaseCursor {
    /// Get the primary key associated with the record at the cursor's current position.
    ///
    /// Returns the first key's primary key if the first cursor result hasn't been retrieved yet.
    #[allow(clippy::missing_errors_doc)]
    pub fn primary_key<T>(&self) -> crate::Result<T>
    where
        T: TryFromJs,
    {
        let key = self.as_sys().primary_key()?;
        T::from_js(key).map_err(Into::into)
    }

    /// [`Self::primary_key`] mirror using `serde`.
    #[allow(clippy::missing_errors_doc)]
    #[cfg(feature = "serde")]
    pub fn primary_key_ser<T>(&self) -> crate::Result<T>
    where
        T: crate::serde::DeserialiseFromJs,
    {
        let key = self.as_sys().primary_key()?;
        T::deserialise_from_js(key)
    }

    /// Advance the cursor by `step` records.
    ///
    /// Equivalent to calling [`advance(step)`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/advance)
    /// in JS.
    #[errdoc(Cursor(TransactionInactiveError, InvalidStateError))]
    #[allow(clippy::missing_errors_doc)]
    pub async fn advance_by(&mut self, step: u32) -> crate::Result<()> {
        if step == 0 {
            Ok(())
        } else {
            self.as_sys().advance(step)?;
            self.req().await
        }
    }

    /// Advance the cursor to the given key. The next record fetched will be for the given key.
    ///
    /// Refers to the key path or generated key when called on an
    /// [`ObjectStore`](crate::object_store::ObjectStore) and the indexed key path when called on
    /// an [`Index`](crate::index::Index).
    ///
    /// Equivalent to calling [`continue(key)`](https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor/continue)
    /// in JS.
    #[errdoc(Cursor(TransactionInactiveError, DataError, InvalidStateError))]
    #[allow(clippy::missing_errors_doc)]
    pub async fn continue_to_key<T>(&mut self, key: T) -> crate::Result<()>
    where
        T: TryToJs,
    {
        let key = key.try_to_js()?;
        self.continue_to_key_common(&key).await
    }

    /// [`Self::continue_to_key`] mirror using `serde`.
    #[allow(clippy::missing_errors_doc)]
    #[cfg(feature = "serde")]
    pub async fn continue_to_key_ser<T>(&mut self, key: T) -> crate::Result<()>
    where
        T: crate::serde::SerialiseToJs,
    {
        let key = key.serialise_to_js()?;
        self.continue_to_key_common(&key).await
    }

    /// Get the cursor's direction
    #[inline]
    #[must_use]
    pub fn direction(&self) -> CursorDirection {
        self.as_sys().direction()
    }

    pub(crate) fn req(&self) -> VoidRequest {
        VoidRequest::new(self.as_sys().req())
    }

    #[must_use]
    pub(crate) fn has_req(&self) -> bool {
        let req = self.as_sys().maybe_req();
        !(req.is_null() || req.is_undefined())
    }

    pub(crate) fn poll_state<R, F>(
        &mut self,
        cx: &mut Context<'_>,
        read_current: F,
    ) -> Poll<crate::Result<Option<R>>>
    where
        F: FnOnce(&mut Self) -> crate::Result<R>,
    {
        let req_poll = match self.state {
            CursorState::ReadingNext(ref mut req) => req.poll_unpinned(cx),
            CursorState::ReadCurrent => return Poll::Ready(self.read_current(read_current)),
            CursorState::TryNext => return self.poll_try_next(cx, read_current),
        };

        self.on_req_polled(req_poll, read_current)
    }

    #[inline]
    pub(super) fn on_cursor_position_reset(&mut self) {
        self.state = CursorState::ReadCurrent;
    }

    #[cfg(feature = "streams")]
    pub(crate) fn is_finished(&self) -> bool {
        matches!(self.state, CursorState::ReadCurrent) && !self.has_req()
    }

    fn read_current<R, F>(&mut self, callback: F) -> crate::Result<Option<R>>
    where
        F: FnOnce(&mut Self) -> crate::Result<R>,
    {
        Ok(if self.has_req() {
            let out = callback(self)?;
            self.state = CursorState::TryNext;
            Some(out)
        } else {
            None
        })
    }

    fn poll_try_next<R, F>(
        &mut self,
        cx: &mut Context<'_>,
        read_current: F,
    ) -> Poll<crate::Result<Option<R>>>
    where
        F: FnOnce(&mut Self) -> crate::Result<R>,
    {
        if let Err(e) = self.as_sys().continue_() {
            return Poll::Ready(Err(e.into()));
        }

        let mut req = self.req();
        let req_poll = req.poll_unpinned(cx);

        self.state = CursorState::ReadingNext(req);
        self.on_req_polled(req_poll, read_current)
    }

    fn on_req_polled<R, F>(
        &mut self,
        poll: Poll<crate::Result<()>>,
        read_current: F,
    ) -> Poll<crate::Result<Option<R>>>
    where
        F: FnOnce(&mut Self) -> crate::Result<R>,
    {
        match poll {
            Poll::Ready(res) => {
                self.state = CursorState::ReadCurrent;

                Poll::Ready(match res {
                    Ok(()) => self.read_current(read_current),
                    Err(e) => Err(e),
                })
            }
            Poll::Pending => Poll::Pending,
        }
    }

    async fn continue_to_key_common(&mut self, key: &JsValue) -> crate::Result<()> {
        self.as_sys().continue_with_key(key)?;
        self.req().await?;
        self.on_cursor_position_reset();
        Ok(())
    }

    pub(super) fn invalidate_current(&mut self) {
        if matches!(self.state, CursorState::ReadCurrent) {
            self.state = CursorState::TryNext;
        }
    }

    pub(crate) fn key_sys(&self) -> crate::Result<JsValue> {
        self.as_sys().key().map_err(Into::into)
    }

    pub(crate) fn value_sys(&self) -> crate::Result<JsValue> {
        self.as_sys().value().map_err(Into::into)
    }
}

impl SystemRepr for BaseCursor {
    type Repr = CursorSys;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.sys
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.sys
    }
}
