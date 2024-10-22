use macroific::elements::ModulePrefix;
use macroific::prelude::*;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{DeriveInput, LitStr, Token, Type};

const FMT: ModulePrefix<2> = ModulePrefix::new(["core", "fmt"]);

pub(crate) struct FromDomException {
    ident: Ident,
    variants: Vec<Variant>,
    has_default: bool,
}

#[derive(AttributeOptions)]
struct VariantOpts {
    default: bool,
}

struct Variant {
    ident: Ident,
    opts: VariantOpts,
}

impl TryFrom<syn::Variant> for Variant {
    type Error = syn::Error;

    fn try_from(var: syn::Variant) -> Result<Self, Self::Error> {
        let opts = VariantOpts::from_iter_named("from_dom_exception", var.span(), var.attrs)?;

        let fields = var.fields.extract_unnamed_fields()?;
        let span = fields.span();

        let mut fields = fields.into_iter();
        let Some(first) = fields.next() else {
            return Err(syn::Error::new(span, "expected a single field"));
        };

        if let Some(next) = fields.next() {
            return Err(syn::Error::new(next.span(), "expected a single field"));
        }

        drop(fields);
        let span = first.span();
        let Type::Path(p) = first.ty else {
            return Err(syn::Error::new(span, "expected a type path"));
        };

        match p.path.segments.last() {
            Some(last) if last.ident == "DomException" => {}
            _ => return Err(syn::Error::new(p.span(), "expected a `DomException`")),
        }

        Ok(Self {
            ident: var.ident,
            opts,
        })
    }
}

impl Parse for FromDomException {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let DeriveInput { ident, data, .. } = input.parse()?;
        let base_variants = data.extract_enum()?.variants;
        let mut variants = Vec::with_capacity(base_variants.len());

        let mut has_default = false;

        for var in base_variants {
            let var = Variant::try_from(var)?;

            if var.opts.default {
                if has_default {
                    return Err(syn::Error::new_spanned(
                        var.ident,
                        "multiple default variants",
                    ));
                }

                has_default = true;
            }

            variants.push(var);
        }

        Ok(Self {
            ident,
            variants,
            has_default,
        })
    }
}

impl FromDomException {
    #[must_use]
    pub fn into_token_stream(self) -> TokenStream {
        let Self {
            ident: enum_ident,
            variants,
            has_default,
        } = self;

        let arms_from_base = variants.iter().filter_map(move |v| {
            if v.opts.default {
                return None;
            }

            let ident = &v.ident;
            let ident_str = LitStr::new(&ident.to_string(), ident.span());
            Some(quote!(#ident_str => Self::#ident(e),))
        });

        let arms_to_base = variants
            .iter()
            .map(|v| {
                let ident = &v.ident;
                quote!(#enum_ident::#ident(e))
            })
            .collect::<Punctuated<_, Token![|]>>();

        let arms_name = variants.iter().map(move |v| {
            let ident = &v.ident;

            if v.opts.default {
                quote!(Self::#ident(e) => return ::std::borrow::Cow::Owned(e.name()))
            } else {
                let ident_str = LitStr::new(&ident.to_string(), ident.span());
                quote!(Self::#ident(_) => #ident_str)
            }
        });

        let arms_fmt = variants.iter().map(move |v| {
            let ident = &v.ident;

            if v.opts.default {
                quote! {
                    Self::#ident(e) => {
                        #FMT::Display::fmt(&e.to_string(), f)?;
                        e.message()
                    }
                }
            } else {
                let ident_str = LitStr::new(&ident.to_string(), ident.span());
                quote! {
                    Self::#ident(e) => {
                        f.write_str(#ident_str)?;
                        e.message()
                    }
                }
            }
        });

        let default = if has_default {
            quote!(Self::Other(e))
        } else {
            quote!(Self::Unknown(e.unchecked_into()))
        };

        quote! {
            #[automatically_derived]
            impl #enum_ident {
                /// Get the exception [name](::web_sys::DomException::name)
                pub fn name(&self) -> std::borrow::Cow<str> {
                    std::borrow::Cow::Borrowed(match self {
                        #(#arms_name),*
                    })
                }
            }

            #[automatically_derived]
            #[::sealed::sealed]
            impl crate::internal_utils::SystemRepr for #enum_ident {
                type Repr = ::web_sys::DomException;

                fn as_sys(&self) -> &Self::Repr {
                    match self {
                        #arms_to_base => e,
                    }
                }

                fn into_sys(self) -> Self::Repr {
                    match self {
                        #arms_to_base => e,
                    }
                }
            }

            #[automatically_derived]
            impl From<::web_sys::DomException> for #enum_ident {
                fn from(e: ::web_sys::DomException) -> Self {
                    match e.name().as_str() {
                        #(#arms_from_base)*
                        _ => #default,
                    }
                }
            }

            #[automatically_derived]
            impl #FMT::Display for #enum_ident {
                fn fmt(&self, f: &mut #FMT::Formatter<'_>) -> #FMT::Result {
                    let msg = match self {
                        #(#arms_fmt),*
                    };
                    f.write_str(": ")?;
                    f.write_str(&msg)
                }
            }
        }
    }
}
