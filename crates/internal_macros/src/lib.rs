use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
mod dynamic_storage;

#[proc_macro_attribute]
pub fn dynamic_storage(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match dynamic_storage::expand(input) {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
