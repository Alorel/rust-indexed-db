use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use internal_macros::callback_bounds;

use crate::error::{OpenDbOpError, CALLBACK_ERRORED};
use crate::internal_utils::{StructName, SystemRepr};
use crate::{Database, OpenDbOpResult, VersionChangeEvent};

use super::VoidRequest;

/// Future for opening a database
#[derive(StructName)]
pub struct OpenDbRequest<B = crate::error::Error, U = crate::error::Error> {
    base: VoidRequest,
    on_blocked: Option<ClosureWrap<B>>,
    on_upgrade_needed: Option<ClosureWrap<U>>,
}

struct ClosureWrap<E> {
    base: Closure<dyn FnMut(web_sys::IdbVersionChangeEvent) -> JsValue + 'static>,
    rx: Receiver<E>,
}

enum Either<B, U> {
    Left(B),
    Right(U),
}

impl<E: 'static> ClosureWrap<E> {
    #[callback_bounds(err(E), fut(Fut), func(F))]
    fn new<F, Fut>(callback: F) -> Self {
        let (tx, rx) = oneshot::channel();
        let base = Closure::once(move |evt: web_sys::IdbVersionChangeEvent| -> JsValue {
            let evt = VersionChangeEvent::new(evt);
            let promise = future_to_promise(async move {
                if let Err(e) = callback(evt).await {
                    let _ = tx.send(e);
                    Err(js_sys::Error::new(CALLBACK_ERRORED).unchecked_into())
                } else {
                    Ok(JsValue::UNDEFINED)
                }
            });

            promise.unchecked_into()
        });

        Self { base, rx }
    }
}

impl<E> SystemRepr for ClosureWrap<E> {
    type Repr = js_sys::Function;

    fn as_sys(&self) -> &Self::Repr {
        self.base.as_ref().unchecked_ref()
    }

    fn into_sys(self) -> Self::Repr {
        self.base.into_js_value().unchecked_into()
    }
}

impl<U: 'static> OpenDbRequest<crate::error::Error, U> {
    #[callback_bounds(err(U), fut(Fut), func(F))]
    pub(crate) fn with_upgrade<F, Fut>(req: VoidRequest, on_upgrade_needed: F) -> Self {
        Self::w_upgrade(req, ClosureWrap::new(on_upgrade_needed))
    }
}

impl<B: 'static> OpenDbRequest<B> {
    #[callback_bounds(err(B), fut(Fut), func(F))]
    pub(crate) fn with_block<F, Fut>(req: VoidRequest, on_upgrade_needed: F) -> Self {
        Self::w_block(req, ClosureWrap::new(on_upgrade_needed))
    }
}

impl<B: 'static, U: 'static> OpenDbRequest<B, U> {
    #[callback_bounds(err(B), fut(FutB), func(FB))]
    #[callback_bounds(err(U), fut(FutU), func(FU))]
    pub(crate) fn new<FB, FU, FutB, FutU>(
        req: VoidRequest,
        on_blocked: FB,
        on_upgrade_needed: FU,
    ) -> Self {
        Self::w_both(
            req,
            ClosureWrap::new(on_blocked),
            ClosureWrap::new(on_upgrade_needed),
        )
    }

    fn w_both(
        req: VoidRequest,
        on_blocked: ClosureWrap<B>,
        on_upgrade_needed: ClosureWrap<U>,
    ) -> Self {
        let base = req.as_sys().unchecked_ref::<web_sys::IdbOpenDbRequest>();
        base.set_onupgradeneeded(Some(on_upgrade_needed.as_sys()));
        base.set_onblocked(Some(on_blocked.as_sys()));

        Self {
            base: req,
            on_blocked: Some(on_blocked),
            on_upgrade_needed: Some(on_upgrade_needed),
        }
    }

    fn w_block(req: VoidRequest, on_blocked: ClosureWrap<B>) -> Self {
        req.as_sys()
            .unchecked_ref::<web_sys::IdbOpenDbRequest>()
            .set_onblocked(Some(on_blocked.as_sys()));

        Self {
            base: req,
            on_blocked: Some(on_blocked),
            on_upgrade_needed: None,
        }
    }

    fn w_upgrade(req: VoidRequest, on_upgrade_needed: ClosureWrap<U>) -> Self {
        req.as_sys()
            .unchecked_ref::<web_sys::IdbOpenDbRequest>()
            .set_onupgradeneeded(Some(on_upgrade_needed.as_sys()));

        Self {
            base: req,
            on_blocked: None,
            on_upgrade_needed: Some(on_upgrade_needed),
        }
    }
}

impl<B, U> OpenDbRequest<B, U> {
    fn try_recv_errors(&mut self) -> Option<Either<B, U>> {
        macro_rules! check {
            ($field: ident, $variant: ident) => {
                if let Some(ref mut $field) = self.$field {
                    if let Ok(e) = $field.rx.try_recv() {
                        return Some(Either::$variant(e));
                    }
                }
            };
        }

        check!(on_upgrade_needed, Right);
        check!(on_blocked, Left);
        None
    }

    pub(super) fn do_poll(&mut self, cx: &mut Context<'_>) -> Poll<OpenDbOpResult<Database, B, U>> {
        match self.base.do_poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(match self.try_recv_errors() {
                Some(Either::Left(e)) => OpenDbOpError::Blocked(e),
                Some(Either::Right(e)) => OpenDbOpError::UpgradeNeeded(e),
                None => OpenDbOpError::System(e.into()),
            })),
            Poll::Ready(Ok(())) => Poll::Ready(match self.try_recv_errors() {
                Some(Either::Left(e)) => Err(OpenDbOpError::Blocked(e)),
                Some(Either::Right(e)) => Err(OpenDbOpError::UpgradeNeeded(e)),
                None => Ok(Database::from_req(self.base.as_sys())),
            }),
        }
    }
}

impl OpenDbRequest {
    pub(crate) fn bare(req: VoidRequest) -> Self {
        Self {
            base: req,
            on_blocked: None,
            on_upgrade_needed: None,
        }
    }
}

impl<B, U> Future for OpenDbRequest<B, U> {
    type Output = OpenDbOpResult<Database, B, U>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.do_poll(cx)
    }
}

impl<B, U> Drop for OpenDbRequest<B, U> {
    fn drop(&mut self) {
        let req = self.as_sys();
        req.set_onblocked(None);
        req.set_onupgradeneeded(None);
    }
}

impl<B, U> Debug for OpenDbRequest<B, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(Self::TYPE_NAME)
            .field("base", &self.base)
            .field("on_blocked", &self.on_blocked.is_some())
            .field("on_upgrade_needed", &self.on_upgrade_needed.is_some())
            .finish()
    }
}

impl<B, U> SystemRepr for OpenDbRequest<B, U> {
    type Repr = web_sys::IdbOpenDbRequest;

    fn as_sys(&self) -> &Self::Repr {
        self.base.as_sys().unchecked_ref()
    }

    fn into_sys(self) -> Self::Repr {
        self.as_sys().clone()
    }
}
