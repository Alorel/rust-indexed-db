use super::IdbRequestRef;

/// An request that resolves to ()
#[derive(Debug)]
pub struct VoidRequest(IdbRequestRef);

impl_void_request!(
    for VoidRequest,
    raw web_sys::IdbRequest,
    ref IdbRequestRef,
    fut super::futures::IdbRequestFuture
);
