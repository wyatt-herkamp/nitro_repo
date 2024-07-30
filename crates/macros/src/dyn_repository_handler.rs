use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Result};
pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, .. } = derive_input;

    let Data::Enum(data_enum) = data else {
        return Err(syn::Error::new(ident.span(), "Expected an enum"));
    };

    let variants: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();
    let result = quote! {
        impl Repository for #ident {
            fn get_storage(&self) -> nr_storage::DynStorage {
                match self {
                    #(
                        #ident::#variants(variant) => variant.get_storage(),
                    )*
                }
            }
            fn get_type(&self) -> &'static str {
                match self {
                    #(
                        #ident::#variants(variant) => variant.get_type(),
                    )*
                }
            }
            fn config_types(&self) -> Vec<String> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.config_types(),
                    )*
                }
            }
            fn reload(&self) {
                match self {
                    #(
                        #ident::#variants(variant) => variant.reload(),
                    )*
                }
            }
        }
    };
    Ok(result)
}
