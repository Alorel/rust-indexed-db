use super::IdbOpenDbRequestRef;

/// An [OpenDbRequest][super::OpenDbRequest] that resolves to ()
#[derive(Debug)]
pub struct VoidOpenDbRequest(IdbOpenDbRequestRef);

impl_void_request!(
    VoidOpenDbRequest,
    web_sys::IdbOpenDbRequest,
    IdbOpenDbRequestRef
);
impl_idb_open_request_like!(VoidOpenDbRequest);
