use super::{TransactionResult, TransactionSys};
use crate::error::{DomException, SimpleValueError, UnexpectedDataError};
use accessory::Accessors;
use tokio::sync::mpsc;
use wasm_bindgen::prelude::*;

type Cb = dyn FnMut() + 'static;
type ErrCb = dyn FnMut(web_sys::Event) + 'static;

#[derive(Accessors)]
#[access(defaults(all(vis(pub(super)))))]
pub(super) struct TxListeners {
    #[access(get, get_mut)]
    transaction: TransactionSys,
    closures: Closures,
}

pub(super) struct Closures {
    rx: mpsc::UnboundedReceiver<TransactionResult>,
    _on_success: Closure<Cb>,
    _on_abort: Closure<Cb>,
    _on_error: Closure<ErrCb>,
}

impl TxListeners {
    pub(super) fn new(transaction: web_sys::IdbTransaction) -> Self {
        Self {
            closures: Closures::new(&transaction),
            transaction: transaction.unchecked_into(),
        }
    }

    pub(super) async fn recv(&mut self) -> TransactionResult {
        if let Some(res) = self.closures.rx.recv().await {
            res
        } else {
            unreachable!("Transaction listener channel closed");
        }
    }

    pub(super) fn free_listeners(&self) {
        self.transaction.set_onerror(None);
        self.transaction.set_oncomplete(None);
        self.transaction.set_onabort(None);
    }
}

impl Closures {
    fn new(transaction: &web_sys::IdbTransaction) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let on_success = Closure::once({
            let tx = tx.clone();
            move || {
                let _ = tx.send(TransactionResult::Ok);
            }
        });
        let on_abort = Closure::once({
            let tx = tx.clone();
            move || {
                let _ = tx.send(TransactionResult::Abort);
            }
        });
        let on_error = Closure::once(move |evt: web_sys::Event| {
            let _ = tx.send(TransactionResult::Err(match evt.target() {
                Some(target) => match target.dyn_into::<web_sys::IdbRequest>() {
                    Ok(req) => match req.error() {
                        Ok(Some(e)) => {
                            let e = DomException::from(e);
                            match e {
                                DomException::AbortError(_) => {
                                    let _ = tx.send(TransactionResult::Abort);
                                    return;
                                }
                                e => e.into(),
                            }
                        }
                        Ok(None) => UnexpectedDataError::NoErrorFound.into(),
                        Err(e) => e.into(),
                    },
                    Err(js) => SimpleValueError::DynCast(js.unchecked_into()).into(),
                },
                None => UnexpectedDataError::NoEventTarget.into(),
            }));
        });

        transaction.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        transaction.set_oncomplete(Some(on_success.as_ref().unchecked_ref()));
        transaction.set_onabort(Some(on_abort.as_ref().unchecked_ref()));

        Self {
            rx,
            _on_success: on_success,
            _on_abort: on_abort,
            _on_error: on_error,
        }
    }
}
