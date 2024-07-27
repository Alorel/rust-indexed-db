use std::ops::RangeTo;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, FnArg, ItemFn, Pat, ReturnType, Token};

pub struct GenerateWith {
    func: ItemFn,
    generated_func: ItemFn,
}

impl Parse for GenerateWith {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const PREFIX_SET: &str = "set_";
        const PREFIX_SET_REPLACE_RANGE: RangeTo<usize> = ..PREFIX_SET.len();
        const REPLACEMENT: &str = "with_";
        const E_BAD_FN: &str = "Expected a &mut self fn";

        let func = input.parse::<ItemFn>()?;
        let mut ident_str = func.sig.ident.to_string();

        if !ident_str.starts_with(PREFIX_SET) {
            return Err(syn::Error::new(
                func.sig.ident.span(),
                "Function name must start with 'set_'",
            ));
        }

        ident_str.replace_range(PREFIX_SET_REPLACE_RANGE, REPLACEMENT);

        let mut generated_func = func.clone();

        match generated_func.sig.inputs.first_mut() {
            Some(FnArg::Receiver(this)) => {
                *this = parse_quote!(mut self);
            }
            Some(o) => return Err(syn::Error::new(o.span(), E_BAD_FN)),
            None => return Err(syn::Error::new(generated_func.sig.span(), E_BAD_FN)),
        }

        generated_func.block = {
            let orig_name = &func.sig.ident;
            let mut args = Vec::with_capacity(func.sig.inputs.len() - 1);

            for fn_arg in func.sig.inputs.iter().skip(1) {
                match fn_arg {
                    FnArg::Typed(pat) => match &*pat.pat {
                        Pat::Ident(pat) => args.push(&pat.ident),
                        _ => return Err(syn::Error::new(fn_arg.span(), "Expected an identifier")),
                    },
                    FnArg::Receiver(_) => {
                        return Err(syn::Error::new(fn_arg.span(), "Expected an identifier"))
                    }
                }
            }

            parse_quote!({
                self.#orig_name(#(#args),*);
                self
            })
        };

        generated_func.sig.output = {
            let arrow = <Token![->]>::default();
            let this = Box::new(parse_quote!(Self));
            ReturnType::Type(arrow, this)
        };
        generated_func.sig.ident = Ident::new(&ident_str, func.sig.ident.span());

        if !generated_func.attrs.iter().any(is_inline_attr) {
            generated_func.attrs.push(syn::parse_quote!(#[inline]));
        }

        Ok(Self {
            func,
            generated_func,
        })
    }
}

impl ToTokens for GenerateWith {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            func,
            generated_func,
        } = self;

        func.to_tokens(tokens);
        generated_func.to_tokens(tokens);
    }
}

fn is_inline_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("inline")
}
