use super::super::traits::*;
use crate::error::UnexpectedDataError;
use cfg_if::cfg_if;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use wasm_bindgen::prelude::*;

type Callback = Closure<dyn FnMut(web_sys::Event) + 'static>;

/// represents the value on an event.target.result
pub(crate) enum EventTargetResult {
    /// the event.target.result was null
    Null,
    /// the event.target.result was a [`web_sys::IdbCursor`] instance
    #[cfg(feature = "cursors")]
    Cursor(crate::cursor::CursorSys),
    /// the event.target.result was not null
    NotNull,
}

pub(super) struct Listeners {
    rx: mpsc::Receiver<EventTargetResult>,
    req: web_sys::IdbRequest,
    _callback: Callback,
}

impl Listeners {
    pub(super) fn new(req: web_sys::IdbRequest) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let callback = Callback::wrap(Box::new(move |e: web_sys::Event| {
            let non_null_result = e
                .target()
                .map(JsValue::from)
                // get the event.target.result
                .and_then(|val| js_sys::Reflect::get(&val, &JsValue::from("result")).ok())
                // make sure its not null or undefined
                .filter(|val| !val.is_undefined() && !val.is_null());

            let _ = tx.try_send(match non_null_result {
                None => EventTargetResult::Null,
                #[cfg_attr(not(feature = "cursors"), expect(unused_variables))]
                Some(val) => {
                    cfg_if! {
                        if #[cfg(feature = "cursors")] {
                             match val.dyn_into::<crate::cursor::CursorSys>() {
                                Ok(cursor) => EventTargetResult::Cursor(cursor),
                                Err(_) => EventTargetResult::NotNull,
                            }
                        } else {
                            EventTargetResult::NotNull
                        }
                    }
                }
            });
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
    type Output = crate::Result<EventTargetResult>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self.rx.poll_recv(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(event_target)) => Poll::Ready(super::UntypedRequest::req_to_result(
                &self.req,
                event_target,
            )),
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
