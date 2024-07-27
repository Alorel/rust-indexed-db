use std::fmt::{Display, Formatter};

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, LitStr, Path};

use crate::commons::AttrStream;

use super::TokenStream1;

pub(crate) struct Serrdoc {
    inherit_path: Option<InheritPath>,
    attr_stream: AttrStream,
}

struct InheritPath {
    fn_name: String,
    struct_path: Path,
}

impl Serrdoc {
    #[inline(always)]
    #[must_use]
    pub fn exec(inherit: TokenStream1, mut target: TokenStream1) -> TokenStream1 {
        match Self::parse_inherit(inherit) {
            Ok(inherit_path) => match syn::parse(target) {
                Ok(attr_stream) => Self {
                    inherit_path,
                    attr_stream,
                }
                .into_token_stream(),
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

    fn parse_inherit(inherit: TokenStream1) -> syn::Result<Option<InheritPath>> {
        if inherit.is_empty() {
            Ok(None)
        } else {
            syn::parse(inherit).map(Some)
        }
    }

    #[must_use]
    fn into_token_stream(self) -> TokenStream {
        const BASE_MSG: &str = " Additionally, an [`Error::Serialisation`](crate::error::Error::Serialisation)([`SerialisationError::Serde`](crate::error::SerialisationError::Serde)) may be thrown.";

        let Self {
            inherit_path,
            mut attr_stream,
        } = self;
        let msg = if let Some(path) = inherit_path {
            let mut path = path.to_string();
            path.push_str(BASE_MSG);
            LitStr::new(&path, Span::call_site())
        } else {
            LitStr::new(BASE_MSG, Span::call_site())
        };

        attr_stream.attrs.push(parse_quote!(#[doc = #msg]));
        attr_stream.into_token_stream()
    }
}

impl Parse for InheritPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input = input.parse::<Path>()?;

        match input.segments.last() {
            Some(last) if input.segments.len() > 1 => Ok(Self {
                fn_name: last.ident.to_string(),
                struct_path: input,
            }),
            _ => Err(syn::Error::new(
                input.span(),
                "Expected a path with >=2 segments",
            )),
        }
    }
}

impl Display for InheritPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            fn_name,
            struct_path,
        } = self;
        let struct_path = struct_path.to_token_stream();
        f.write_fmt(format_args!(
            " # Errors\nSee [`{fn_name}`]({struct_path}::{fn_name})."
        ))
    }
}
