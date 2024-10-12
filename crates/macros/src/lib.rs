pub(crate) mod dyn_repository_handler;
pub(crate) mod nu_type;
pub(crate) mod repository_config;
pub(crate) mod serde;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};
pub(crate) mod scopes;
#[proc_macro_derive(RepositoryConfig, attributes(repository_config))]
pub fn repository_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    let result = repository_config::expand(input);
    match result {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(DynRepositoryHandler, attributes(repository_handler))]
pub fn dyn_repository_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    let result = dyn_repository_handler::expand(input);
    match result {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Scopes, attributes(scope))]
pub fn scopes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    let result = scopes::expand(input);
    match result {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(SerdeViaStr)]
pub fn serde_via_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match serde::expand(input) {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
#[proc_macro_derive(NuType)]
pub fn nu_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match nu_type::expand(input) {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
