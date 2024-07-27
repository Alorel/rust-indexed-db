use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::sync::mpsc;
use wasm_bindgen::prelude::*;

use crate::internal_utils::StructName;
use crate::internal_utils::SystemRepr;

type TClosure = Closure<dyn FnMut() + 'static>;
type Rx<T> = mpsc::UnboundedReceiver<Result<T, web_sys::DomException>>;
type Tx<T> = mpsc::UnboundedSender<Result<T, web_sys::DomException>>;

/// Alias for [`Request<()>`].
pub type VoidRequest = Request<()>;

/// Future for a [request](IdbRequest)
#[derive(StructName)]
pub struct Request<T = JsValue> {
    req: web_sys::IdbRequest,
    rx: Rx<T>,
    _on_success: TClosure,
    _on_error: TClosure,
}

impl VoidRequest {
    pub(crate) fn void(req: web_sys::IdbRequest) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let on_error = make_on_error(tx.clone(), req.clone());
        let on_success = Closure::once(move || {
            let _ = tx.send(Ok(()));
        });

        Self::new(req, rx, on_success, on_error)
    }
}

impl Request {
    pub(crate) fn jsval(req: web_sys::IdbRequest) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let on_error = make_on_error(tx.clone(), req.clone());
        let on_success = Closure::once({
            let req = req.clone();
            move || {
                let _ = tx.send(Ok(req.result().unwrap()));
            }
        });

        Self::new(req, rx, on_success, on_error)
    }
}

impl<T> Request<T> {
    fn new(req: web_sys::IdbRequest, rx: Rx<T>, on_success: TClosure, on_error: TClosure) -> Self {
        set_listeners(&req, &on_success, &on_error);

        Self {
            req,
            rx,
            _on_success: on_success,
            _on_error: on_error,
        }
    }

    pub(super) fn do_poll(&mut self, cx: &mut Context<'_>) -> Poll<crate::Result<T>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(res)) => Poll::Ready(res.map_err(Into::into)),
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => unreachable!(),
        }
    }
}

impl<T> Future for Request<T> {
    type Output = crate::Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(cx)
    }
}

impl<T> Drop for Request<T> {
    #[inline]
    fn drop(&mut self) {
        drop_listeners(&self.req);
    }
}

impl<T> Debug for Request<T> {
    struct_name_debug!(inner self, self.as_sys());
}

impl<T> SystemRepr for Request<T> {
    type Repr = web_sys::IdbRequest;

    #[inline]
    fn as_sys(&self) -> &Self::Repr {
        &self.req
    }

    #[inline]
    fn into_sys(self) -> Self::Repr {
        self.req.clone()
    }
}

fn make_on_error<T: 'static>(tx: Tx<T>, req: web_sys::IdbRequest) -> TClosure {
    Closure::once(move || {
        let _ = tx.send(Err(req.error().unwrap().unwrap()));
    })
}

fn set_listeners(req: &web_sys::IdbRequest, on_success: &TClosure, on_error: &TClosure) {
    req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
    req.set_onerror(Some(on_error.as_ref().unchecked_ref()));
}

fn drop_listeners(req: &web_sys::IdbRequest) {
    req.set_onsuccess(None);
    req.set_onerror(None);
}
