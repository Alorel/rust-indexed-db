use web_sys::DomException;

use super::IdbTransactionResult;

#[derive(Debug, Clone)]
pub(crate) enum InternalTxResult {
    Success,
    Error,
    Abort,
}

impl InternalTxResult {
    pub fn to_external(&self, tx: &web_sys::IdbTransaction) -> IdbTransactionResult {
        match self {
            InternalTxResult::Success => IdbTransactionResult::Success,
            InternalTxResult::Error => IdbTransactionResult::Error(extract_error(tx)),
            InternalTxResult::Abort => IdbTransactionResult::Abort,
        }
    }
}

fn extract_error(tx: &web_sys::IdbTransaction) -> DomException {
    tx.error().expect("Failed to unwrap transaction error")
}
