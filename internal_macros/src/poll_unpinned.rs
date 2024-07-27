use macroific::elements::ModulePrefix;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, DeriveInput, Generics};

const POLL_UNPINNED: ModulePrefix<'static, 3> =
    ModulePrefix::new(["crate", "future", "PollUnpinned"]).with_leading_sep(false);

#[allow(clippy::module_name_repetitions)]
pub struct DeriveFutureFromPollUnpinned {
    ident: Ident,
    generics: Generics,
}

impl Parse for DeriveFutureFromPollUnpinned {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let DeriveInput {
            ident, generics, ..
        } = input.parse()?;

        Ok(Self { ident, generics })
    }
}

impl DeriveFutureFromPollUnpinned {
    pub fn derive(input: crate::TokenStream1) -> crate::TokenStream1 {
        let DeriveFutureFromPollUnpinned { ident, generics } = syn::parse_macro_input!(input);

        let generics_mut = {
            let mut generics = generics.clone();
            generics.params.push(parse_quote!(__out));
            generics
        };
        let (g1, _, _) = generics_mut.split_for_impl();
        let (_, g2, g3) = generics.split_for_impl();

        let out = quote! {
            #[automatically_derived]
            impl #g1 ::std::future::Future for #ident #g2 #g3
            where
              Self: #POLL_UNPINNED<Output = __out> + ::std::marker::Unpin,
            {
                type Output = __out;

                #[inline]
                fn poll(mut self: ::std::pin::Pin<&mut Self>, cx: &mut ::std::task::Context) -> ::std::task::Poll<Self::Output> {
                    #POLL_UNPINNED::poll_unpinned(&mut *self, cx)
                }
            }
        };

        out.into()
    }
}
