use super::{CursorDirection, CursorSys};
use crate::future::request::listeners::EventTargetResult;
use crate::future::{PollUnpinned, Request};
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
    ReadingNext(Request<EventTargetResult>),
}

impl BaseCursor {
    /// Get the primary key associated with the record at the cursor's current position.
    ///
    /// # Returns
    ///
    /// - The first key's primary key if the first cursor result hasn't been retrieved yet.
    /// - `None` if the cursor's iterated past its end
    #[allow(clippy::missing_errors_doc)]
    pub fn primary_key<T>(&self) -> crate::Result<Option<T>>
    where
        Option<T>: TryFromJs,
    {
        let key = self.as_sys().primary_key()?;
        TryFromJs::from_js(key).map_err(Into::into)
    }

    /// [`Self::primary_key`] mirror using `serde`.
    #[allow(clippy::missing_errors_doc)]
    #[cfg(feature = "serde")]
    pub fn primary_key_ser<T>(&self) -> crate::Result<Option<T>>
    where
        Option<T>: crate::serde::DeserialiseFromJs,
    {
        let key = self.as_sys().primary_key()?;
        crate::serde::DeserialiseFromJs::deserialise_from_js(key)
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
            self.req().await.map(|_| ())
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

    pub(crate) fn req(&self) -> Request<EventTargetResult> {
        Request::new(self.as_sys().req())
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
        !self.has_key() || (matches!(self.state, CursorState::ReadCurrent) && !self.has_req())
    }

    fn has_req(&self) -> bool {
        let req = self.as_sys().maybe_req();
        !req.is_undefined() && !req.is_null()
    }

    fn has_key(&self) -> bool {
        match self.as_sys().key() {
            Ok(k) => !k.is_undefined() && !k.is_null(),
            Err(_) => false,
        }
    }

    fn read_current<R, F>(&mut self, callback: F) -> crate::Result<Option<R>>
    where
        F: FnOnce(&mut Self) -> crate::Result<R>,
    {
        // Chrome implementation: request gets unset on cursor end
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
        poll: Poll<crate::Result<EventTargetResult>>,
        read_current: F,
    ) -> Poll<crate::Result<Option<R>>>
    where
        F: FnOnce(&mut Self) -> crate::Result<R>,
    {
        match poll {
            Poll::Ready(res) => {
                self.state = CursorState::ReadCurrent;

                let should_continue = res.map(|event_result| match event_result {
                    EventTargetResult::Null => false,
                    EventTargetResult::Cursor(cursor_sys) => {
                        self.sys = cursor_sys;
                        true
                    }
                    EventTargetResult::NotNull => true,
                });

                Poll::Ready(match should_continue {
                    Ok(true) => {
                        // Firefox implementation: key gets set to undefined on cursor end
                        if self.has_key() {
                            self.read_current(read_current)
                        } else {
                            Ok(None)
                        }
                    }
                    // Chrome implementation: the only way to know if a cursor has finished
                    // is by reading the value from onsuccess event.target.result
                    Ok(false) => Ok(None),
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

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for BaseCursor {
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
