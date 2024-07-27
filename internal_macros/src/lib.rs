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
#![allow(rustdoc::private_intra_doc_links)]

use proc_macro::TokenStream as TokenStream1;

use quote::ToTokens;
use syn::parse_macro_input;

mod commons;
mod errdoc;
mod from_dom_exception;
mod struct_name;

mod build_into_fut;
mod generate_with;
mod generic_bounds;
mod poll_unpinned;
mod struct_name_debug;

/// Add consistent generic bounds across the crate. See [options](generic_bounds::Opts).
#[proc_macro_attribute]
pub fn generic_bounds(spec: TokenStream1, target: TokenStream1) -> TokenStream1 {
    generic_bounds::exec(spec, target)
}

/// Generate a `with_` setter for a `set_` setter
#[proc_macro_attribute]
pub fn generate_with(_args: TokenStream1, input: TokenStream1) -> TokenStream1 {
    parse_macro_input!(input as generate_with::GenerateWith)
        .into_token_stream()
        .into()
}

/// Delegate debug to inner type, wrap with type name.
#[proc_macro_derive(StructNameDebug, attributes(debug))]
pub fn derive_struct_name_debug(input: TokenStream1) -> TokenStream1 {
    struct_name_debug::StructNameDebug::derive(input)
}

/// Derive a [`Future`](std::future::Future) from a `PollUnpinned`.
#[proc_macro_derive(FutureFromPollUnpinned)]
pub fn derive_future_from_poll_unpinned(input: TokenStream1) -> TokenStream1 {
    poll_unpinned::DeriveFutureFromPollUnpinned::derive(input)
}

/// Derive an `IntoFuture` for variations of the type that implement `BuildPrimitive`.
#[proc_macro_derive(BuildIntoFut)]
pub fn derive_primitive_into_fut(input: TokenStream1) -> TokenStream1 {
    parse_macro_input!(input as build_into_fut::BuildIntoFut)
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
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```
/// #[errdoc(
///     InvalidStateError("Thrown when bars do a foo."),
/// )]
/// ````
#[proc_macro_attribute]
pub fn errdoc(defs: TokenStream1, function: TokenStream1) -> TokenStream1 {
    errdoc::Errdoc::exec(defs, function)
}
