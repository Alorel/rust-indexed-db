use crate::prelude::*;
use accessory::Accessors;
use delegate_display::DelegateFmt;
use derive_more::{Deref, DerefMut, From};
use fancy_constructor::new;
use idb_fut::database::Database;
use idb_fut::KeyPath;
use impartial_ord::ImpartialOrd;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::iter;
use std::iter::FusedIterator;
use std::ops::RangeInclusive;

#[derive(Copy, Clone, Default, Ord, ImpartialOrd, DelegateFmt, Deref, DerefMut, From, new)]
#[dfmt(ddisplay, ddebug)]
#[new(const_fn)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct Key(i8);

#[derive(Copy, Clone, Default, Ord, ImpartialOrd, DelegateFmt, Deref, DerefMut, From, new)]
#[dfmt(ddisplay, ddebug)]
#[new(const_fn)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct Value(u8);

#[derive(Copy, Clone, Eq, PartialEq, Default, Ord, ImpartialOrd, new, Accessors)]
#[access(get, set, defaults(get(const_fn, cp)))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyVal {
    #[new(into)]
    key: Key,

    #[new(into)]
    value: Value,
}

impl Key {
    #[allow(unused)]
    pub const PATH: &'static str = "key";

    #[allow(unused)]
    pub const KEY_PATH: KeyPath<&'static str> = KeyPath::One(Self::PATH);

    pub const MIN: Self = Self(-10);
    pub const MAX: Self = Self(-1);

    #[allow(unused)]
    pub fn iter_range() -> iter::Map<RangeInclusive<i8>, fn(i8) -> Self> {
        (*Self::MIN..=*Self::MAX).map(Self::new)
    }
}

impl Value {
    #[allow(unused)]
    pub const PATH: &'static str = "value";

    #[allow(unused)]
    pub const KEY_PATH: KeyPath<&'static str> = KeyPath::One(Self::PATH);

    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(10);

    #[allow(unused)]
    pub fn iter_range() -> iter::Map<RangeInclusive<u8>, fn(u8) -> Self> {
        (*Self::MIN..=*Self::MAX).map(Self::new)
    }
}

impl KeyVal {
    pub const RANGE_LEN: u32 = (Value::MAX.0 + 1 - Value::MIN.0) as u32;

    pub async fn insert_keyval_docs(db: &Database) {
        open_tx!(db, Readwrite > (tx, store));

        for record in Self::iter_range() {
            if let Err(e) = store.add(record).build_dyn() {
                panic!("Error inserting {record:?}: {e}");
            }
        }

        drop(store);
        tx.commit().await.expect("commit");
    }

    pub fn iter_range() -> impl FusedIterator<Item = Self> + DoubleEndedIterator + ExactSizeIterator
    {
        (*Key::MIN..=*Key::MAX).map(move |k| Self::new(Key::new(k), Value::new(-k as u8)))
    }
}

impl Debug for KeyVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&(self.key(), self.value()), f)
    }
}

macro_rules! impl_eq {
    ($($outer: ident ($inner: ident)),+) => {$(
       impl Eq for $outer {}
       impl PartialEq<$inner> for $outer {
           #[inline]
           fn eq(&self, other: &$inner) -> bool {
             other.eq(&self.0)
           }
       }

       impl PartialEq for $outer {
           #[inline]
           fn eq(&self, other: &$outer) -> bool {
             self.0 == other.0
           }
       }
    )+};
}

macro_rules! impl_math {
    ($($outer: ident ($inner: ident)),+) => {$(
        impl ::std::ops::Add<$inner> for $outer {
            type Output = Self;

            #[inline]
            fn add(self, rhs: $inner) -> Self {
                Self(self.0 + rhs)
            }
        }

        impl ::std::ops::Add for $outer {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self {
                self + rhs.0
            }
        }

        impl ::std::ops::Sub<$inner> for $outer {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: $inner) -> Self {
                Self(self.0 - rhs)
            }
        }

        impl ::std::ops::Sub for $outer {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                self - rhs.0
            }
        }

        impl ::std::ops::Mul<$inner> for $outer {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: $inner) -> Self {
                Self(self.0 * rhs)
            }
        }

        impl ::std::ops::Mul for $outer {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self {
                self * rhs.0
            }
        }
    )+};
}

impl_eq!(Key(i8), Value(u8));
impl_math!(Key(i8), Value(u8));

#[cfg(not(feature = "serde"))]
const _: () = {
    use crate::prelude::*;
    use idb_fut::error::SimpleValueError;
    use js_sys::Reflect;

    macro_rules! primitives {
        ($($ty: ident),+ $(,)?) => {
          $(
              impl TryFromJs for $ty {
                fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
                   TryFromJs::from_js(js).map(Self)
                }
              }

              impl TryToJs for $ty {
                #[inline]
                fn try_to_js(&self) -> idb_fut::Result<JsValue> {
                   TryToJs::try_into_js(self.0)
                }
              }
          )+
        };
    }

    primitives!(Key, Value);

    impl TryFromJs for KeyVal {
        fn from_js(js: JsValue) -> Result<Self, SimpleValueError> {
            let key = match Reflect::get(&js, &JsValue::from_str(Key::PATH)) {
                Ok(k) => k,
                Err(e) => return Err(SimpleValueError::DynCast(e)),
            };
            let value = match Reflect::get(&js, &JsValue::from_str(Value::PATH)) {
                Ok(v) => v,
                Err(e) => return Err(SimpleValueError::DynCast(e)),
            };

            Ok(Self {
                key: Key::from_js(key)?,
                value: Value::from_js(value)?,
            })
        }
    }

    impl TryToJs for KeyVal {
        fn try_to_js(&self) -> idb_fut::Result<JsValue> {
            let obj = js_sys::Object::new().unchecked_into::<JsValue>();
            Reflect::set(&obj, &JsValue::from_str(Key::PATH), &self.key.try_to_js()?)?;
            Reflect::set(
                &obj,
                &JsValue::from_str(Value::PATH),
                &self.value.try_to_js()?,
            )?;

            Ok(obj)
        }
    }
};
