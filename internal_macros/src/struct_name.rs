use macroific::elements::{GenericImpl, ModulePrefix};
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
        const TRAIT_NAME: ModulePrefix<'static, 3> =
            ModulePrefix::new(["crate", "internal_utils", "StructName"]).with_leading_sep(false);

        let Self { ident, generics } = self;

        let str = LitStr::new(&ident.to_string(), ident.span());
        let header = GenericImpl::new(generics)
            .with_trait(TRAIT_NAME)
            .with_target(ident);

        quote! {
            #header {
                const TYPE_NAME: &'static str = #str;
            }
        }
    }
}
