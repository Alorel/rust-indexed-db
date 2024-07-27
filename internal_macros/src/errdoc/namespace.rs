use syn::{parse_quote, Attribute};

trait TNamespaceBase: Sized {
    #[must_use]
    fn try_from_str(str: &str) -> Option<Self>;

    /// `[name, message]`
    #[must_use]
    fn name_and_msg(self) -> [&'static str; 2];

    #[must_use]
    fn msg(self) -> &'static str {
        self.name_and_msg()[1]
    }
}

trait TNamespace: Sized {
    fn name_and_msg(self) -> [&'static str; 2];

    fn into_attr(self) -> Attribute {
        let [name, msg] = self.name_and_msg();
        doc_attr_line(name, msg)
    }
}

fn doc_attr_line(name: &str, desc: &str) -> Attribute {
    let msg = format!(" | [`{name}`](crate::error::DomException::{name}) | {desc} |");
    parse_quote!(#[doc = #msg])
}

macro_rules! impl_namespaces {
    ($container: ident ($($namespace: ident ( $($variant: ident ($msg: literal)),+ $(,)? )),+ $(,)?)) => {
        pub(super) enum $container {
            $($namespace(syn::punctuated::Punctuated<$namespace, syn::token::Comma>),)+
        }

        $(
            #[derive(Copy, Clone)]
            #[repr(u8)]
            #[allow(clippy::enum_variant_names)]
            pub(super) enum $namespace {
                $($variant,)+
            }
        )+

        impl ::syn::parse::Parse for $container {
            fn parse(input: syn::parse::ParseStream) -> ::syn::Result<Self> {
                let ident = input.parse::<proc_macro2::Ident>()?;
                let ident_str = ident.to_string();

                let contents;
                syn::parenthesized!(contents in input);

                match ident_str.as_str() {
                    $(
                        stringify!($namespace) => {
                            syn::punctuated::Punctuated::parse_terminated(&contents).map($container::$namespace)
                        },
                    )+
                    _ => Err(syn::Error::new(syn::spanned::Spanned::span(&ident), "Unrecognised namespace")),
                }
            }
        }

        impl $container {
            pub fn extend_attrs(self, attrs: &mut Vec<syn::Attribute>) {
                match self {
                    $(
                        $container::$namespace(ns) => {
                            attrs.extend(ns.into_iter().map(TNamespace::into_attr));
                        },
                    )+
                }
            }
        }

        $(
            impl TNamespaceBase for $namespace {
                fn name_and_msg(self) -> [&'static str; 2] {
                    match self {
                        $(Self::$variant => [stringify!($variant), $msg],)+
                    }
                }

                fn try_from_str(str: &str) -> Option<Self> {
                    match str {
                        $(stringify!($variant) => Some(Self::$variant),)+
                        _ => None
                    }
                }
            }

            impl ::syn::parse::Parse for $namespace {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    let ident = input.parse::<proc_macro2::Ident>()?;

                    match <$namespace as TNamespaceBase>::try_from_str(&ident.to_string()) {
                        Some(ns) => Ok(ns),
                        None => Err(syn::Error::new(syn::spanned::Spanned::span(&ident), "Unrecognised alias")),
                    }
                }
            }
        )+
    };
}

macro_rules! name_msg {
    (fwd ($($ty: ty),+ $(,)?) ) => {
        $(impl TNamespace for $ty {
            #[inline]
            fn name_and_msg(self) -> [&'static str; 2] {
                <Self as TNamespaceBase>::name_and_msg(self)
            }
        })+
    };
    ($ty: ident {
        $($variant: ident => $name: literal),+ $(,)?
    }) => {
        impl TNamespace for $ty {
            fn name_and_msg(self) -> [&'static str; 2] {
                match self {
                    $(Self::$variant => [$name, self.msg()],)+
                    o => TNamespaceBase::name_and_msg(o),
                }
            }
        }
    };
}

impl_namespaces!(Namespace(
    QuerySource(
        InvalidStateError("Thrown if this object store or index has been deleted."),
        TransactionInactiveError("Thrown if the transaction associated with this operation is inactive."),
        DataError("Thrown if the key or key range provided contains an invalid key."),
        ConstraintError("Thrown if an object store is already using the specified `name`."),
    ),
    Database(
        TransactionInactiveError("Thrown if a request is made on a source database that does not exist (for example, when the database has been deleted or removed)."),
        ConstraintError("Thrown if an object store with the given name (based on a case-sensitive comparison) already exists in the connected database."),

        InvalidStateErrorObjectStore("Thrown if the method was not called from a [`Versionchange`](crate::transaction::TransactionMode::Versionchange) transaction callback."),
        NotFoundErrorDeleteObjectStore("Thrown when trying to delete an object store that does not exist."),
        InvalidAccessErrorCreateObjectStore("Thrown if autoIncrement is set to true and keyPath is either an empty string or an array containing an empty string."),

        NotFoundErrorTx("Thrown if an object store specified cannot be found."),
        InvalidAccessErrorTx("Thrown if the fn was called with an empty list of store names."),
    ),
    ObjectStore(
        ReadOnlyError("Thrown if the transaction associated with this operation is in [`Readonly`](crate::transaction::TransactionMode::Readonly) mode."),
        TransactionInactiveError("Thrown if the transaction associated with this operation is inactive."),
        DataErrorAdd("See [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/add#dataerror)."),
        DataErrorDelete("Thrown if the key is not a valid key or key range."),
        InvalidStateError("Thrown if the object store has been deleted or removed."),
        DataCloneError("Thrown if the data being stored could not be cloned by the internal structured cloning algorithm."),
        ConstraintError("Thrown if an insert operation failed because the primary key constraint was violated (due to an already existing record with the same primary key value)."),
    ),
    Transaction(
        NotFoundError("Thrown if the requested object store is not in this transaction's scope."),
        InvalidStateError("Thrown if the request was made on a source object that has been deleted or removed, or if the transaction has finished."),
    ),
    Index(
        ConstraintError("Thrown if an index with the same name already exists in the database. Index names are case-sensitive."),
        InvalidAccessError("Thrown if the provided key path is a sequence, and [`multi_entry`](crate::index::IndexParameters::multi_entry) is set to true."),
        InvalidStateError("Thrown if the method was not called from a [`VersionChange`](crate::transaction::TransactionMode::Versionchange) transaction mode callback or the object store has been deleted."),
        InvalidStateErrorIndex("Thrown if the object store has been deleted or removed, or if the transaction has already finished."),
        SyntaxError("Thrown if the provided keyPath is not a [valid key path](https://www.w3.org/TR/IndexedDB/#dfn-valid-key-path)."),
        TransactionInactiveError("Thrown if the transaction this object store belongs to is not active (e.g. has been deleted or removed)."),
        NotFoundError("Thrown if there is no index with the given name (case-sensitive) in the database."),
    ),
    Cursor(
        TransactionInactiveError("Thrown if the transaction associated with this cursor is inactive."),
        DataErrorOpen("Thrown if the key or key range provided contains an invalid key."),
        InvalidStateErrorOpen("Thrown if the object store or index has been deleted."),
        InvalidStateError("Thrown if the cursor is currently being iterated or has iterated past its end."),
        DataError("Thrown if the key is invalid, is less than the cursor's current position (if the cursor is moving forward) or is greater than the cursor's current position (if the cursor is moving backward)."),
        DataErrorUpdate("Thrown if the underlying object store uses in-line keys and the property in the value at the object store's key path does not match the key in this cursor's position."),
        InvalidAccessError("Thrown if the cursor's direction is not [`Prev`](crate::cursor::CursorDirection::Prev) or [`Next`](crate::cursor::CursorDirection::Next)."),
        ReadOnlyError("Thrown if the transaction mode is [`Readonly`](crate::transaction::TransactionMode::Readonly)."),
        DataCloneError("Thrown if the data being stored could not be cloned by the internal structured cloning algorithm.")
    ),
));

name_msg!(fwd(QuerySource, Transaction));

name_msg!(Cursor {
    InvalidStateErrorOpen => "InvalidStateError",
    DataErrorOpen => "DataError",
    DataErrorUpdate => "DataError",
});

name_msg!(ObjectStore {
    DataErrorAdd => "DataError",
    DataErrorDelete => "DataError",
});

name_msg!(Database {
    NotFoundErrorDeleteObjectStore => "NotFoundError",
    InvalidAccessErrorCreateObjectStore => "InvalidAccessError",
    InvalidStateErrorObjectStore => "InvalidStateError",
    NotFoundErrorTx => "NotFoundError",
    InvalidAccessErrorTx => "InvalidAccessError",
});

name_msg!(Index {
    InvalidStateErrorIndex => "InvalidStateError",
});
