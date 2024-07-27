use super::BaseCursor;
use crate::future::{BasicRequest, VoidRequest};
use crate::internal_utils::SystemRepr;
use crate::primitive::{TryFromJs, TryToJs};
use fancy_constructor::new;
use internal_macros::BuildIntoFut;
use sealed::sealed;
use std::marker::PhantomData;

pub struct None;

/// Builder for [`update`](super::Cursor::update).
#[derive(new, BuildIntoFut)]
#[new(vis(pub(super)))]
#[must_use]
pub struct Update<'a, T, KT = None> {
    cur: &'a BaseCursor,
    value: T,

    #[new(val(PhantomData))]
    key_type: PhantomData<KT>,
}

impl<'a, T, KT> Update<'a, T, KT> {
    /// Set the type of the key to be returned.
    #[inline]
    pub fn with_key_type<KT2>(self) -> Update<'a, T, KT2>
    where
        KT2: TryFromJs + Unpin,
    {
        Update::new(self.cur, self.value)
    }
}

#[sealed]
impl<T: TryToJs> crate::BuildPrimitive for Update<'_, T> {
    type Fut = VoidRequest;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let Self {
            cur,
            value,
            key_type: _,
        } = self;

        let js = value.try_to_js()?;
        let req = cur.as_sys().update(&js)?;

        Ok(VoidRequest::new(req))
    }
}

#[sealed]
impl<T: TryToJs, KT: TryFromJs + Unpin> crate::BuildPrimitive for Update<'_, T, KT> {
    type Fut = BasicRequest<KT>;

    fn primitive(self) -> crate::Result<Self::Fut> {
        let Self {
            cur,
            value,
            key_type: _,
        } = self;

        let js = value.try_to_js()?;
        let req = cur.as_sys().update(&js)?;

        Ok(BasicRequest::new_primitive(req))
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use serde::{de::DeserializeOwned, Serialize};

    #[sealed]
    impl<T: Serialize> crate::BuildSerde for Update<'_, T> {
        type Fut = VoidRequest;

        fn serde(self) -> crate::Result<Self::Fut> {
            let Self {
                cur,
                value,
                key_type: _,
            } = self;

            let js = serde_wasm_bindgen::to_value(&value)?;
            let req = cur.as_sys().update(&js)?;

            Ok(VoidRequest::new(req))
        }
    }

    #[sealed]
    impl<T: Serialize, KT: DeserializeOwned + Unpin> crate::BuildSerde for Update<'_, T, KT> {
        type Fut = BasicRequest<KT>;

        fn serde(self) -> crate::Result<Self::Fut> {
            let Self {
                cur,
                value,
                key_type: _,
            } = self;

            let js = serde_wasm_bindgen::to_value(&value)?;
            let req = cur.as_sys().update(&js)?;

            Ok(BasicRequest::new_ser(req))
        }
    }
};
