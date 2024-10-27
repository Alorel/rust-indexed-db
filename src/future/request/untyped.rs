use super::super::traits::*;
use super::Listeners;
use sealed::sealed;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use web_sys::IdbRequestReadyState;

pub(super) enum UntypedRequest {
    Bare(web_sys::IdbRequest),
    WithListeners(Listeners),
}

#[::sealed::sealed]
#[allow(unused_qualifications)]
impl crate::internal_utils::SystemRepr for UntypedRequest {
    type Repr = web_sys::IdbRequest;

    fn as_sys(&self) -> &Self::Repr {
        match self {
            Self::Bare(req) => req,
            Self::WithListeners(listeners) => listeners.as_sys(),
        }
    }

    fn into_sys(self) -> Self::Repr {
        match self {
            Self::Bare(req) => req,
            Self::WithListeners(listeners) => listeners.into_sys(),
        }
    }
}

impl UntypedRequest {
    pub(super) fn req_to_result(req: &web_sys::IdbRequest) -> crate::Result<()> {
        match req.error() {
            Ok(None) => Ok(()),
            Ok(Some(e)) => Err(e.into()),
            Err(e) => Err(e.into()),
        }
    }

    fn poll_request(req: &web_sys::IdbRequest) -> Poll<crate::Result<()>> {
        if matches!(req.ready_state(), IdbRequestReadyState::Done) {
            Poll::Ready(Self::req_to_result(req))
        } else {
            Poll::Pending
        }
    }

    fn create_listeners(req: &mut web_sys::IdbRequest) -> Listeners {
        // Take without cloning
        let req = std::mem::replace(req, JsValue::NULL.unchecked_into());
        Listeners::new(req)
    }
}

#[sealed]
impl PollUnpinned for UntypedRequest {
    type Output = crate::Result<()>;

    fn poll_unpinned(&mut self, cx: &mut Context) -> Poll<Self::Output> {
        match self {
            Self::WithListeners(listeners) => listeners.poll_unpinned(cx),
            Self::Bare(req) => {
                if let Poll::Ready(res) = Self::poll_request(req) {
                    Poll::Ready(res)
                } else {
                    let mut listeners = Self::create_listeners(req);
                    let out = listeners.poll_unpinned(cx);

                    *self = Self::WithListeners(listeners);

                    out
                }
            }
        }
    }
}
