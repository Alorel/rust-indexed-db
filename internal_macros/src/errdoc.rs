use macroific::elements::Attributed;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Token};

use namespace::Namespace;

use crate::TokenStream1;

mod namespace;

type PNamespace = Punctuated<Namespace, Token![,]>;

pub(crate) struct Errdoc {
    namespaces: PNamespace,
    attr_stream: Attributed,
}

#[repr(transparent)]
struct DefVec(PNamespace);

impl Parse for DefVec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        PNamespace::parse_terminated(input).map(Self)
    }
}

impl Errdoc {
    #[inline]
    #[must_use]
    pub fn exec(defs: TokenStream1, mut function: TokenStream1) -> TokenStream1 {
        match syn::parse::<DefVec>(defs) {
            Ok(defs) => match syn::parse(function) {
                Ok(attr_stream) => Self {
                    namespaces: defs.0,
                    attr_stream,
                }
                .into_token_stream(),
                Err(e) => e.into_compile_error(),
            }
            .into(),
            Err(e) => {
                let e = TokenStream1::from(e.into_compile_error());
                function.extend(e);
                function
            }
        }
    }

    fn into_token_stream(self) -> TokenStream {
        let Self {
            namespaces,
            mut attr_stream,
        } = self;

        attr_stream.attributes.extend([
            parse_quote!(#[doc = "\n # Errors\n"]),
            parse_quote!(#[doc = " | Error | Thrown when |"]),
            parse_quote!(#[doc = " |---|---|"]),
        ]);

        for namespace in namespaces {
            namespace.extend_attrs(&mut attr_stream.attributes);
        }

        attr_stream.into_token_stream()
    }
}
