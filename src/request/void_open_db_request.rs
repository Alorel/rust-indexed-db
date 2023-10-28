use super::IdbOpenDbRequestRef;

/// An [`OpenDbRequest`](super::OpenDbRequest) that resolves to ()
#[derive(Debug)]
pub struct VoidOpenDbRequest(IdbOpenDbRequestRef);

impl_void_request!(
    for VoidOpenDbRequest,
    raw web_sys::IdbOpenDbRequest,
    ref IdbOpenDbRequestRef,
    fut super::futures::IdbOpenDbRequestFuture
);
impl_idb_open_request_like!(VoidOpenDbRequest);
