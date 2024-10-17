use macroific::elements::module_prefix::RESULT;
use macroific::elements::ModulePrefix;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, DeriveInput, GenericParam, Generics, WherePredicate};

const UNPIN: ModulePrefix<'static, 3> = ModulePrefix::new(["core", "marker", "Unpin"]);
const BUILD: ModulePrefix<'static, 3> =
    ModulePrefix::new(["crate", "build", "Build"]).with_leading_sep(false);

const MB_ERR: ModulePrefix<'static, 3> =
    ModulePrefix::new(["crate", "future", "MaybeErrored"]).with_leading_sep(false);

pub(crate) struct BuildIntoFut {
    ident: Ident,

    generics: Generics,
}

impl Parse for BuildIntoFut {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let DeriveInput {
            ident, generics, ..
        } = input.parse()?;
        Ok(Self { ident, generics })
    }
}

impl BuildIntoFut {
    pub fn into_token_stream(self) -> TokenStream {
        let Self { ident, generics } = self;
        let mut gen2 = generics.clone();

        gen2.params.extend::<[GenericParam; 3]>([
            parse_quote!(_pFut),
            parse_quote!(_pOut),
            parse_quote!(_pErr),
        ]);

        gen2.make_where_clause().predicates.extend::<[WherePredicate; 3]>([
            parse_quote!(Self: #BUILD<Ok = _pFut, Err = _pErr>),
            parse_quote!(_pFut: crate::future::PollUnpinned<Output = #RESULT<_pOut, _pErr>> + #UNPIN),
            parse_quote!(_pErr: #UNPIN)
        ]);

        let (_, g2, _) = generics.split_for_impl();
        let (g1, _, g3) = gen2.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #g1 ::std::future::IntoFuture for #ident #g2 #g3 {
                type Output = #RESULT<_pOut, _pErr>;
                type IntoFuture = #MB_ERR<_pFut, _pErr>;

                #[inline]
                fn into_future(self) -> Self::IntoFuture {
                    match #BUILD::build(self) {
                        Ok(fut) => #MB_ERR::running(fut),
                        Err(e) => #MB_ERR::errored(e),
                    }
                }
            }
        }
    }
}
