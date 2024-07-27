//! Internal macros for the [indexed_db_futures](https://crates.io/crates/indexed_db_futures) crate. The API is to be
//! considered unstable and subject to breaking changes at any time, although these will follow semver.

#![deny(clippy::correctness, clippy::suspicious)]
#![warn(
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    missing_docs
)]

use proc_macro::TokenStream as TokenStream1;

use quote::ToTokens;
use syn::parse_macro_input;

mod callback_bounds;
mod commons;
mod errdoc;
mod from_dom_exception;
mod struct_name;

mod generate_with;
mod serrdoc;

/// Add future & closure type bounds to a fn/impl
#[proc_macro_attribute]
pub fn callback_bounds(spec: TokenStream1, target: TokenStream1) -> TokenStream1 {
    callback_bounds::CallbackBounds::exec(spec, target)
}

/// Generate a `with_` setter for a `set_` setter
#[proc_macro_attribute]
pub fn generate_with(_args: TokenStream1, input: TokenStream1) -> TokenStream1 {
    parse_macro_input!(input as generate_with::GenerateWith)
        .into_token_stream()
        .into()
}

/// Derive `StructName`
#[proc_macro_derive(StructName)]
pub fn derive_struct_name(input: TokenStream1) -> TokenStream1 {
    parse_macro_input!(input as struct_name::StructName)
        .into_token_stream()
        .into()
}

/// Implement `From<DomException>`
#[proc_macro_derive(dom_exception_err, attributes(from_dom_exception))]
pub fn derive_from_dom_exception(input: TokenStream1) -> TokenStream1 {
    parse_macro_input!(input as from_dom_exception::FromDomException)
        .into_token_stream()
        .into()
}

/// Document the errors this fn returns
///
/// # Example
///
/// ```rust
/// #[errdoc(
///     InvalidStateError("Thrown when bars do a foo."),
/// )]
/// ```
#[proc_macro_attribute]
pub fn errdoc(defs: TokenStream1, function: TokenStream1) -> TokenStream1 {
    errdoc::Errdoc::exec(defs, function)
}

/// Add a doc re: a serde error being thrown.
#[proc_macro_attribute]
pub fn serrdoc(inherit: TokenStream1, target: TokenStream1) -> TokenStream1 {
    serrdoc::Serrdoc::exec(inherit, target)
}
