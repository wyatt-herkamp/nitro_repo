use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Result;
pub(crate) fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let Data::Struct(struct_data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "NuType can only be derived for structs",
        ));
    };
    // TODO: Non String nu types
    if struct_data.fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            input,
            "NuType can only be derived for structs with a single field",
        ));
    }

    let field = struct_data.fields.iter().next().unwrap();

    let (field_ref, other_ref, value) = if let Some(ident) = &field.ident {
        (
            quote! { &self.#ident },
            quote! { &other.#ident },
            quote! { value.#ident },
        )
    } else {
        (quote! { &self.0 }, quote! { &other.0 }, quote! { value.0 })
    };
    let result = quote! {
        const _: () = {
            impl std::ops::Deref for #name {
                type Target = str;
                fn deref(&self) -> &Self::Target {
                    #field_ref
                }
            }
            impl std::convert::AsRef<str> for #name {
                fn as_ref(&self) -> &str {
                    #field_ref
                }
            }
            impl std::cmp::PartialEq for #name {
                fn eq(&self, other: &Self) -> bool {
                   #field_ref == #other_ref
                }
            }
            impl std::cmp::PartialEq<str> for #name {
                fn eq(&self, other: &str) -> bool {
                    #field_ref == other
                }
            }
            impl std::cmp::PartialEq<#name> for str {
                fn eq(&self, other: &#name) -> bool {
                    self == #other_ref
                }
            }
            impl std::cmp::PartialEq<&str> for #name {
                fn eq(&self, other: &&str) -> bool {
                    #field_ref  == *other
                }
            }
            impl std::cmp::PartialEq<#name> for &str {
                fn eq(&self, other: &#name) -> bool {
                    *self == #other_ref
                }
            }
            impl std::cmp::Eq for #name {}

            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(#field_ref, f)
                }
            }
            impl std::convert::From<#name> for String {
                fn from(value: #name) -> Self {
                    #value
                }
            }
        };
    };
    Ok(result)
}
