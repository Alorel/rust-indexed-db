//! Futures in use by the crate

pub use count::CountRequest;
pub use maybe_errored::MaybeErrored;
pub use open_db::OpenDbRequest;
pub use primitive::PrimitiveRequest;
pub use request::{Request, VoidRequest};

macro_rules! struct_name_debug {
    (inner $self: ident, $field: expr) => {
        fn fmt(&$self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_tuple(<Self as crate::internal_utils::StructName>::TYPE_NAME)
                .field($field)
                .finish()
        }
    };
    ($ty: ty, $field: expr) => {
        impl ::std::fmt::Debug for $ty {
            struct_name_debug!(inner $field);
        }
    };
}

mod count;
mod maybe_errored;
mod open_db;
mod primitive;
mod request;

iffeat! {
    #[cfg(feature = "serde")]
    mod serde;
    pub use serde::SerdeRequest;
}

iffeat! {
    #[cfg(feature = "list-databases")]
    mod list_dbs;
    pub use list_dbs::ListDatabasesFuture;
}
