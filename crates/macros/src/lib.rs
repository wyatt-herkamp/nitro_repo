pub(crate) mod dyn_repository_handler;
pub(crate) mod repository_config;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

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

#[proc_macro_derive(DynRepositoryHandler)]
pub fn dyn_repository_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    let result = dyn_repository_handler::expand(input);
    match result {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}