use crate::error::SimpleValueError;

/// A two-way branching [`TryToJs`](super::TryToJs)/[`TryFromJs`](super::TryFromJs) enum.
///
/// To be used, for example, when retrieving records from an [`ObjectStore`](crate::object_store::ObjectStore) that has
/// different types of values via [`get_all`](crate::query_source::QuerySource::get_all).
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
#[allow(missing_docs)]
pub enum Switch2<A, B> {
    A(A),
    B(B),
}

/// A three-way branching [`TryToJs`](super::TryToJs)/[`TryFromJs`](super::TryFromJs) enum.
///
/// To be used, for example, when retrieving records from an [`ObjectStore`](crate::object_store::ObjectStore) that has
/// different types of values via [`get_all`](crate::query_source::QuerySource::get_all).
#[allow(missing_docs)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Switch3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

/// A four-way branching [`TryToJs`](super::TryToJs)/[`TryFromJs`](super::TryFromJs) enum.
///
/// To be used, for example, when retrieving records from an [`ObjectStore`](crate::object_store::ObjectStore) that has
/// different types of values via [`get_all`](crate::query_source::QuerySource::get_all).
#[allow(missing_docs)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Switch4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}

/// A five-way branching [`TryToJs`](super::TryToJs)/[`TryFromJs`](super::TryFromJs) enum.
///
/// To be used, for example, when retrieving records from an [`ObjectStore`](crate::object_store::ObjectStore) that
/// has different types of values via [`get_all`](crate::query_source::QuerySource::get_all).
#[allow(missing_docs)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Switch5<A, B, C, D, E> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
}

macro_rules! from {
    ($enum: ident <$($generic: ident),+>, $js: ident $impl: block) => {
        #[allow(unused_qualifications)]
        impl<$($generic),+> super::from_js::TryFromJs for $enum<$($generic),+>
        where
            $($generic: super::TryFromJs,)+
        {
            fn from_js($js: ::wasm_bindgen::JsValue) -> Result<Self, $crate::error::SimpleValueError> $impl
        }
    };
    ($js: expr, $variant: ident, |$e: ident| $else: expr) => {
        match <$variant as super::TryFromJs>::from_js($js) {
            Ok(v) => Ok(Self::$variant(v)),
            Err($e) => $else,
        }
    };
}

macro_rules! to {
    ($enum: ident <$($generic: ident),+>) => {
        #[allow(unused_qualifications)]
        impl<$($generic),+> super::try_to_js::TryToJs for $enum<$($generic),+>
        where
            $($generic: super::TryToJs,)+
        {
            fn try_to_js(&self) -> crate::Result<::wasm_bindgen::JsValue> {
                match self {
                    $(Self::$generic(v) => super::TryToJs::try_to_js(v),)+
                }
            }
        }
    };
}

to!(Switch2<A, B>);
from!(Switch2<A, B>, js {
    from!(js.clone(), A, |e_a| {
        from!(js, B, |e_b| Err(SimpleValueError::Switch(vec![e_a, e_b])))
    })
});

to!(Switch3<A, B, C>);
from!(Switch3<A, B, C>, js {
    from!(js.clone(), A, |e_a| {
        from!(js.clone(), B, |e_b| {
            from!(js, C, |e_c| Err(SimpleValueError::Switch(vec![e_a, e_b, e_c])))
        })
    })
});

to!(Switch4<A, B, C, D>);

from!(Switch4<A, B, C, D>, js {
    from!(js.clone(), A, |e_a| {
        from!(js.clone(), B, |e_b| {
            from!(js.clone(), C, |e_c| {
                from!(js, D, |e_d| Err(SimpleValueError::Switch(vec![e_a, e_b, e_c, e_d])))
            })
        })
    })
});

to!(Switch5<A, B, C, D, E>);

from!(Switch5<A, B, C, D, E>, js {
    from!(js.clone(), A, |e_a| {
        from!(js.clone(), B, |e_b| {
            from!(js.clone(), C, |e_c| {
                from!(js.clone(), D, |e_d| {
                    from!(js, E, |e_e| Err(SimpleValueError::Switch(vec![e_a, e_b, e_c, e_d, e_e])))
                })
            })
        })
    })
});
