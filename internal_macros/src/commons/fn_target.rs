use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Generics, ItemFn, ItemImpl};

pub(crate) enum FnTarget {
    Fn(ItemFn),
    Impl(ItemImpl),
}

impl Parse for FnTarget {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<ItemFn>().is_ok() {
            input.parse().map(FnTarget::Fn)
        } else {
            input.parse().map(FnTarget::Impl)
        }
    }
}

impl FnTarget {
    #[must_use]
    pub fn generics_mut(&mut self) -> &mut Generics {
        match self {
            Self::Fn(v) => &mut v.sig.generics,
            Self::Impl(v) => &mut v.generics,
        }
    }
}

impl ToTokens for FnTarget {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Fn(v) => v.to_tokens(tokens),
            Self::Impl(v) => v.to_tokens(tokens),
        }
    }

    fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Fn(v) => v.into_token_stream(),
            Self::Impl(v) => v.into_token_stream(),
        }
    }
}
