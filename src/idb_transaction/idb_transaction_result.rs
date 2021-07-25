use web_sys::DomException;

/// The [transaction's][crate::idb_transaction::IdbTransaction] result
#[derive(Debug, Clone)]
pub enum IdbTransactionResult {
    /// Transaction committed successfully
    Success,
    /// Transaction errored
    Error(DomException),
    /// Transaction aborted
    Abort,
}

impl IdbTransactionResult {
    /// Convert the transaction into a [Result]
    pub fn into_result(self) -> Result<(), DomException> {
        match self {
            IdbTransactionResult::Success => Ok(()),
            IdbTransactionResult::Error(xc) => Err(xc),
            IdbTransactionResult::Abort => {
                let xc = DomException::new_with_message("Transaction aborted")
                    .expect("Failed to construct abort event dom exception");
                Err(xc)
            }
        }
    }
}
