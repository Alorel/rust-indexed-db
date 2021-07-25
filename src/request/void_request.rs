use super::IdbRequestRef;

/// An request that resolves to ()
#[derive(Debug)]
pub struct VoidRequest(IdbRequestRef);

impl_void_request!(VoidRequest, web_sys::IdbRequest, IdbRequestRef);
