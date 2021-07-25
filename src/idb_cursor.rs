//! Cursor-related code
//!
//! Features required: `cursors`

use std::future::Future;
use std::rc::Rc;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DomException, IdbCursorDirection};

pub use idb_cursor_with_value::*;

use crate::idb_query_source::IdbQuerySource;
use crate::internal_utils::optional_jsvalue_undefined;
use crate::request::{
    IdbCursorAdvancementFuture, IdbRequestFuture, IdbRequestRef, JsCastRequestFuture, VoidRequest,
};

mod idb_cursor_with_value;

/// An interface for an IndexedDB cursor
///
/// Features required: `cursors`
#[derive(Debug)]
pub struct IdbCursor<'a, T: IdbQuerySource> {
    inner: web_sys::IdbCursor,
    source: &'a T,
    req: Rc<IdbRequestRef>,
}

impl<'a, T: IdbQuerySource> IdbCursor<'a, T> {
    #[inline]
    pub(crate) fn new(inner: web_sys::IdbCursor, source: &'a T, req: Rc<IdbRequestRef>) -> Self {
        Self { inner, source, req }
    }

    #[inline]
    pub(crate) fn inner_as_cursor_with_value(&self) -> &web_sys::IdbCursorWithValue {
        self.inner.unchecked_ref()
    }

    /// Get what spawned this cursor
    #[inline]
    pub fn source(&self) -> &'a T {
        &self.source
    }

    /// Get the cursor direction
    #[inline]
    pub fn direction(&self) -> IdbCursorDirection {
        self.inner.direction()
    }

    /// Get the key at the cursor's current position. Returns `None` if the cursor is outside its
    /// range.
    pub fn key(&self) -> Option<JsValue> {
        optional_jsvalue_undefined(self.inner.key().unwrap())
    }

    /// Get the cursor's current effective primary key. Returns `None` if the cursor is currently
    /// being iterated or has iterated outside its range.
    #[inline]
    pub fn primary_key(&self) -> Option<JsValue> {
        optional_jsvalue_undefined(self.inner.primary_key().unwrap())
    }

    /// Common code for continue methods
    fn continue_common(&self) -> IdbCursorAdvancementFuture {
        let fut = IdbRequestFuture::new_with_rc(self.req.clone(), true);
        IdbCursorAdvancementFuture::new(fut)
    }

    /// Advances the cursor to the next position along its direction
    pub fn continue_cursor(
        &self,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner.continue_()?;
        Ok(self.continue_common())
    }

    /// Advances the cursor to the next position along its direction, to the item whose key matches
    /// the given key parameter
    pub fn continue_cursor_with_key<K: JsCast>(
        &self,
        key: &K,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner.continue_with_key(key.unchecked_ref())?;
        Ok(self.continue_common())
    }

    /// Sets the cursor to the given index key and primary key given as arguments.
    pub fn continue_primary_key<K: JsCast, PK: JsCast>(
        &self,
        key: &K,
        primary_key: &PK,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner
            .continue_primary_key(key.unchecked_ref(), primary_key.unchecked_ref())?;
        Ok(self.continue_common())
    }

    /// Sets the number of times a cursor should move its position forward.
    pub fn advance(
        &self,
        count: u32,
    ) -> Result<impl Future<Output = Result<bool, DomException>>, DomException> {
        self.inner.advance(count)?;
        Ok(self.continue_common())
    }

    /// Delete the record at the cursor's position, without changing the cursor's position
    pub fn delete(&self) -> Result<VoidRequest, DomException> {
        Ok(VoidRequest::new(self.inner.delete()?))
    }

    /// Update the value at the current position of the cursor in the object store
    pub fn update<V: JsCast>(
        &self,
        value: &V,
    ) -> Result<impl Future<Output = Result<JsValue, DomException>>, DomException> {
        JsCastRequestFuture::new(self.inner.update(value.unchecked_ref()))
    }
}
