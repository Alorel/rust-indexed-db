use super::super::traits::*;
use crate::error::UnexpectedDataError;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use wasm_bindgen::prelude::*;

type Callback = Closure<dyn FnMut() + 'static>;

pub(super) struct Listeners {
    rx: mpsc::Receiver<()>,
    req: web_sys::IdbRequest,
    _callback: Closure<dyn FnMut() + 'static>,
}

impl Listeners {
    pub(super) fn new(req: web_sys::IdbRequest) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let callback = Callback::wrap(Box::new(move || {
            let _ = tx.try_send(());
        }));

        let as_fn = callback.as_ref().unchecked_ref();
        req.set_onsuccess(Some(as_fn));
        req.set_onerror(Some(as_fn));

        Self {
            rx,
            req,
            _callback: callback,
        }
    }
}

impl Drop for Listeners {
    fn drop(&mut self) {
        self.req.set_onsuccess(None);
        self.req.set_onerror(None);
    }
}

#[::sealed::sealed]
impl PollUnpinned for Listeners {
    type Output = crate::Result<()>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.rx.poll_recv(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(())) => Poll::Ready(super::UntypedRequest::req_to_result(&self.req)),
            Poll::Ready(None) => Poll::Ready(Err(UnexpectedDataError::ChannelDropped.into())),
        }
    }
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for Listeners {
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
