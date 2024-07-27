use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::Attribute;

pub(crate) struct AttrStream {
    pub attrs: Vec<Attribute>,
    rest: TokenStream,
}

impl Parse for AttrStream {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: Attribute::parse_outer(input)?,
            rest: input.parse()?,
        })
    }
}

impl ToTokens for AttrStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { attrs, rest } = self;

        for attr in attrs {
            attr.to_tokens(tokens);
        }

        rest.to_tokens(tokens);
    }
}
