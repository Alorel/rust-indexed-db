use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Generics, LitStr};

pub struct StructName {
    ident: Ident,
    generics: Generics,
}

impl Parse for StructName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input = input.parse::<DeriveInput>()?;

        Ok(Self {
            generics: input.generics,
            ident: input.ident,
        })
    }
}

impl StructName {
    #[must_use]
    pub fn into_token_stream(self) -> TokenStream {
        let Self { ident, generics } = self;
        let (g1, g2, g3) = generics.split_for_impl();
        let str = LitStr::new(&ident.to_string(), ident.span());

        quote! {
            impl #g1 crate::internal_utils::StructName for #ident #g2 #g3 {
                const TYPE_NAME: &'static str = #str;
            }
        }
    }
}
