use crate::commons::FnTarget;
use crate::TokenStream1;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Error, WherePredicate};

#[inline]
pub(super) fn exec(spec: TokenStream1, target: TokenStream1) -> TokenStream1 {
    match syn::parse::<Opts>(spec) {
        Ok(opts) => match syn::parse::<FnTarget>(target.clone()) {
            Ok(mut target) => {
                opts.extend_target(&mut target);
                target.into_token_stream().into()
            }
            Err(e) => on_err(target, e),
        },
        Err(e) => on_err(target, e),
    }
}

macro_rules! make_opts {
    ($struct_name: ident => { $($($opt: ident)|+ => $predicate: ty),+ $(,)? }) => {
        /// Options list
        ///
        /// | Option | Predicate |
        /// |--------|-----------|
        $($(#[doc = concat!(" | `", stringify!($opt), "` | `", stringify!($predicate), "` |")])+)+
        #[derive(::macroific::attr_parse::AttributeOptions)]
        pub(super) struct $struct_name {
            $($($opt: ::syn::punctuated::Punctuated<proc_macro2::Ident, ::syn::Token![,]>),+),+
        }

        impl ::syn::parse::Parse for $struct_name {
            #[inline]
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                ::macroific::attr_parse::AttributeOptions::from_stream(input)
            }
        }

        impl $struct_name {
            fn extend_target(self, target: &mut FnTarget) {
                $($(if !self.$opt.is_empty() {
                    extend_generics(self.$opt, target, ::quote::quote!($predicate));
                })+)+
            }
        }
    };
}

make_opts!(Opts => {
    db_name|index_name|store_name|key_path => ::core::convert::AsRef<str>,
    db_version => crate::factory::DBVersion,
    blocked_cb => ::core::ops::FnOnce(crate::database::VersionChangeEvent) -> crate::Result<()> + 'static,
    upgrade_cb => ::core::ops::FnOnce(crate::database::VersionChangeEvent, crate::database::Database) -> crate::Result<()> + 'static,
});

fn on_err(mut target: TokenStream1, e: Error) -> TokenStream1 {
    let e: TokenStream1 = e.into_compile_error().into();
    target.extend(e);
    target
}

fn extend_generics<T, I, P>(idents: Punctuated<I, P>, target: &mut FnTarget, ext_with: T)
where
    T: ToTokens,
    I: ToTokens,
{
    let iter = idents
        .into_iter()
        .map(move |id| -> WherePredicate { parse_quote!(#id: #ext_with) });

    target
        .generics_mut()
        .make_where_clause()
        .predicates
        .extend(iter);
}
