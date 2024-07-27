use crate::error::Error;
use sealed::sealed;
use std::future::Future;

/// Finalise the builder.
///
/// Implemented:
///
/// 1. Directly when the return type is neither primitive nor `serde`-serialisable.
/// 2. As an alias for [`BuildPrimitive`] for types that implement it.
#[sealed(pub(crate))]
pub trait Build {
    /// Successful response
    type Ok;

    /// Unsuccessful response
    type Err;

    /// Finalise the builder
    #[allow(clippy::missing_errors_doc)]
    fn build(self) -> Result<Self::Ok, Self::Err>;
}

/// Finalise the builder for returning a primitive result.
///
/// Types that implement this also implement [`IntoFuture`](std::future::IntoFuture) as an alias for calling
/// [`primitive`](Self::primitive).
#[sealed(pub(crate))]
pub trait BuildPrimitive {
    /// The future returned.
    type Fut: Future;

    /// Finalise the builder for returning a primitive result.
    ///
    /// Types that implement this also implement [`IntoFuture`](std::future::IntoFuture) as an alias for calling
    /// this fn.
    #[allow(clippy::missing_errors_doc)]
    fn primitive(self) -> crate::Result<Self::Fut>;
}

/// Finalise the builder for returning a [deserialisable](serde::Deserialize) result.
#[sealed(pub(crate))]
#[cfg(feature = "serde")]
pub trait BuildSerde {
    /// The future returned.
    type Fut: Future;

    /// Finalise the builder for returning a [deserialisable](serde::Deserialize) result.
    #[allow(clippy::missing_errors_doc)]
    fn serde(self) -> crate::Result<Self::Fut>;
}

#[sealed]
impl<T: BuildPrimitive> Build for T {
    type Ok = T::Fut;
    type Err = Error;

    #[inline]
    fn build(self) -> crate::Result<Self::Ok> {
        self.primitive()
    }
}
