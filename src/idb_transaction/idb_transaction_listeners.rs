use std::ops::Deref;
use std::task::Poll;
use std::{
    cell::RefCell,
    rc::Rc,
    task::{Context, Waker},
};

use wasm_bindgen::{prelude::*, JsCast};

use internal_result::InternalTxResult;

use crate::internal_utils::{create_lazy_ref_cell, wake};

use super::IdbTransactionResult;

mod internal_result;

type Cb = dyn Fn() + 'static;
type WakerRef = Rc<RefCell<Option<Waker>>>;
type ResultRef = Rc<RefCell<Option<InternalTxResult>>>;

/// IdbTransaction event listeners
#[derive(Debug)]
pub(crate) struct IdbTransactionListeners {
    waker: WakerRef,
    result: ResultRef,
    on_success: Closure<Cb>,
    on_abort: Closure<Cb>,
    on_error: Closure<Cb>,
}

impl IdbTransactionListeners {
    pub fn new(inner: &web_sys::IdbTransaction) -> Self {
        let waker = create_lazy_ref_cell();
        let result = create_lazy_ref_cell();

        let on_success = create_callback(waker.clone(), result.clone(), InternalTxResult::Success);
        let on_error = create_callback(waker.clone(), result.clone(), InternalTxResult::Error);
        let on_abort = create_callback(waker.clone(), result.clone(), InternalTxResult::Abort);

        inner.set_oncomplete(Some(on_success.as_ref().unchecked_ref()));
        inner.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        inner.set_onabort(Some(on_abort.as_ref().unchecked_ref()));

        Self {
            waker,
            result,
            on_error,
            on_success,
            on_abort,
        }
    }

    pub fn do_poll(
        &self,
        tx: &web_sys::IdbTransaction,
        ctx: &Context<'_>,
    ) -> Poll<IdbTransactionResult> {
        if let Some(v) = self.result.borrow().deref() {
            Poll::Ready(v.to_external(tx))
        } else {
            self.waker.borrow_mut().replace(ctx.waker().clone());
            Poll::Pending
        }
    }
}

fn create_callback(waker: WakerRef, result: ResultRef, kind: InternalTxResult) -> Closure<Cb> {
    let b = Box::new(move || {
        result.borrow_mut().replace(kind.clone());
        wake(&waker);
    });
    Closure::wrap(b)
}
