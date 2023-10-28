use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll, Waker};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DomException, IdbRequestReadyState};

use crate::internal_utils::{create_lazy_ref_cell, wake, NightlyUnwrap};

use super::super::IdbRequestRef;

type WakerRef = Rc<RefCell<Option<Waker>>>;
type ResultRef = Rc<RefCell<Option<OutputResult>>>;
type OutputResult = Result<Option<JsValue>, DomException>;
type Cb = Closure<dyn Fn() + 'static>;

/// Base `IdbRequest` future implementation
#[derive(Debug)]
pub struct IdbRequestFuture {
    request: Rc<IdbRequestRef>,
    result: ResultRef,
    waker: WakerRef,
    _on_success: Option<Cb>,
    on_error: Option<Cb>,
}

impl IdbRequestFuture {
    pub(crate) fn new_with_rc(request: Rc<IdbRequestRef>, read_response: bool) -> Self {
        let waker = create_lazy_ref_cell();
        let result;

        let on_success;
        let on_error;

        // Just set the result if the request has already finished
        if let IdbRequestReadyState::Done = request.inner().ready_state() {
            on_success = None;
            on_error = None;
            let res = extract_success_result(&request, read_response);
            result = Rc::new(RefCell::new(Some(res)));
        } else {
            // Else set error+success listeners
            result = create_lazy_ref_cell();
            let on_success_cb = create_success_closure(
                waker.clone(),
                result.clone(),
                request.clone(),
                read_response,
            );
            let on_error_cb = create_error_closure(waker.clone(), result.clone(), request.clone());

            request
                .inner()
                .set_onsuccess(Some(on_success_cb.as_ref().unchecked_ref()));
            request
                .inner()
                .set_onerror(Some(on_error_cb.as_ref().unchecked_ref()));

            on_success = Some(on_success_cb);
            on_error = Some(on_error_cb);
        }

        Self {
            request,
            result,
            waker,
            _on_success: on_success,
            on_error,
        }
    }

    /// Obtain a weak reference to the request
    #[inline]
    pub(crate) fn weak_request(&self) -> Weak<IdbRequestRef> {
        Rc::downgrade(&self.request)
    }

    /// Obtain a strong reference to the request
    #[inline]
    #[cfg(feature = "cursors")]
    pub(crate) fn strong_request(&self) -> Rc<IdbRequestRef> {
        self.request.clone()
    }

    /// Actual [Future] polling function
    pub(crate) fn do_poll(&self, ctx: &Context<'_>) -> Poll<OutputResult> {
        if self.result.borrow().is_some() {
            let result = self.result.replace(None).nightly_unwrap();
            Poll::Ready(result)
        } else {
            RefCell::borrow_mut(&self.waker).replace(ctx.waker().clone());
            Poll::Pending
        }
    }
}

impl Future for IdbRequestFuture {
    type Output = OutputResult;

    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(ctx)
    }
}

impl Drop for IdbRequestFuture {
    fn drop(&mut self) {
        if self.on_error.is_some() {
            self.request.inner().set_onerror(None);
            self.request.inner().set_onsuccess(None);
        }
    }
}

/// Extract the request result. The Ok result will be `Some` if `read` is true and `None` if it's
/// false
fn extract_success_result(request: &IdbRequestRef, read: bool) -> OutputResult {
    Ok(if read { Some(request.result()?) } else { None })
}

/// Create `on_success` callback
fn create_success_closure(
    waker: WakerRef,
    result: ResultRef,
    request: Rc<IdbRequestRef>,
    read: bool,
) -> Cb {
    let b = Box::new(move || {
        result.replace(Some(extract_success_result(&request, read)));
        wake(&waker);
    });
    Closure::wrap(b)
}

/// Create `on_error` callback
fn create_error_closure(waker: WakerRef, result: ResultRef, request: Rc<IdbRequestRef>) -> Cb {
    let b = Box::new(move || {
        result.replace(Some(Err(request.error().expect("Failed to unwrap error"))));
        wake(&waker);
    });
    Closure::wrap(b)
}
