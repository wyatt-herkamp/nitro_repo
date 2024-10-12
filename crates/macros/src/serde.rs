use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Result;
pub(crate) fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;

    let result = quote! {
        const _: () = {
            impl serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let as_string = self.to_string();
                    as_string.serialize(serializer)
                }
            }
            impl<'de> serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let string = String::deserialize(deserializer)?;
                    Self::try_from(string).map_err(serde::de::Error::custom)
                }
            }
        };
    };
    Ok(result)
}
