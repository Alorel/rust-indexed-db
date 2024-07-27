use macroific::elements::{GenericImpl, ModulePrefix};
use macroific::prelude::*;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, Generics, LitStr};

pub(crate) const ATTR_NAME: &str = "debug";
const FMT: ModulePrefix<'static, 2> = ModulePrefix::new(["std", "fmt"]);

pub struct StructNameDebug {
    ident: Ident,
    generics: Generics,
    field_expr: TokenStream,
}

#[derive(AttributeOptions)]
struct ContainerOpts {
    expr: Option<Expr>,
}

impl Parse for StructNameDebug {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let syn::DeriveInput {
            ident,
            generics,
            data,
            attrs,
            ..
        } = input.parse()?;

        let ContainerOpts { expr: field_expr } =
            ContainerOpts::from_iter_named(ATTR_NAME, Span::call_site(), attrs)?;

        let field_expr = if let Some(expr) = field_expr {
            expr.into_token_stream()
        } else {
            find_field(data.extract_struct_fields()?)?
        };

        Ok(Self {
            ident,
            generics,
            field_expr,
        })
    }
}

impl StructNameDebug {
    #[inline]
    pub fn derive(tokens: crate::TokenStream1) -> crate::TokenStream1 {
        let StructNameDebug {
            ident,
            generics,
            field_expr,
        } = parse_macro_input!(tokens);

        let ident_str = LitStr::new(&ident.to_string(), ident.span());
        let header = GenericImpl::new(generics)
            .with_trait(quote!(#FMT::Debug))
            .with_target(ident);

        let out = quote! {
            #[automatically_derived]
            #header {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.debug_tuple(#ident_str).field(#field_expr).finish()
                }
            }
        };

        out.into()
    }
}

fn find_field<It>(fields: It) -> syn::Result<TokenStream>
where
    It: IntoIterator<Item = syn::Field>,
{
    let mut first = None;
    for (i, field) in fields.into_iter().enumerate() {
        if !field
            .attrs
            .iter()
            .any(move |a| a.path().is_ident(ATTR_NAME))
        {
            continue;
        }

        if first.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "multiple fields flagged with `debug`",
            ));
        }

        first = Some((i, field));
    }

    if let Some((i, field)) = first {
        let mut out = quote!(&self.);

        if let Some(ident) = field.ident {
            out.append(ident);
        } else {
            out.append(Literal::usize_unsuffixed(i));
        }

        Ok(out)
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "no field marked with `debug`",
        ))
    }
}
