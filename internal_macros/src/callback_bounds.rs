use macroific::prelude::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Token, WherePredicate};

use crate::commons::FnTarget;

use crate::TokenStream1;

pub(crate) struct CallbackBounds;

#[derive(AttributeOptions)]
struct Spec {
    fut: Punctuated<Ident, Token![,]>,

    func: Punctuated<Ident, Token![,]>,

    #[attr_opts(default = false)]
    err: Ident,
}

impl Parse for Spec {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let out = Self::from_stream(input)?;
        if out.func.len() == out.fut.len() {
            Ok(out)
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "number of futures and functions must match",
            ))
        }
    }
}

impl CallbackBounds {
    #[inline(always)]
    #[must_use]
    pub fn exec(spec: TokenStream1, mut target: TokenStream1) -> TokenStream1 {
        match syn::parse::<Spec>(spec) {
            Ok(spec) => match syn::parse::<FnTarget>(target) {
                Ok(target) => spec.into_token_stream(target),
                Err(e) => e.into_compile_error(),
            }
            .into(),
            Err(e) => {
                let e = TokenStream1::from(e.into_compile_error());
                target.extend(e);
                target
            }
        }
    }
}

impl Spec {
    #[must_use]
    fn into_token_stream(self, mut target: FnTarget) -> TokenStream {
        let Self { fut, func, err } = self;

        let where_clause = target.generics_mut().make_where_clause();
        let where_err = parse_quote!(#err: Unpin + 'static);
        let iter = fut.into_iter()
            .zip(func.into_iter())
            .flat_map(move |(fut, func)| -> [WherePredicate; 2] {
                let t_fn = parse_quote!(#func: FnOnce(crate::VersionChangeEvent) -> #fut + Unpin + 'static);
                let t_fut = parse_quote! {
                    #fut: ::std::future::Future<Output = Result<(), #err>> + Unpin + 'static
                };

                [t_fn, t_fut]
            })
            .chain(Some(where_err));
        where_clause.predicates.extend(iter);
        target.into_token_stream()
    }
}
