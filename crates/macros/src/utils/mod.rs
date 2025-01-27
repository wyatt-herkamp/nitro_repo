use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Expr, Ident, Lit, LitStr};
pub fn doc_attr_to_string(attr: &Attribute) -> syn::Result<String> {
    match &attr.meta {
        syn::Meta::NameValue(syn::MetaNameValue { value, .. }) => match value {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(lit_str) => Ok(lit_str.value()),
                _ => Err(syn::Error::new_spanned(lit, "Expected a string literal")),
            },
            _ => Err(syn::Error::new_spanned(value, "Expected a string literal")),
        },
        _ => Err(syn::Error::new_spanned(
            &attr.meta,
            "Expected a string literal",
        )),
    }
}
pub struct DisplayStringEnum<'ident, 'entries, T: StringEnum> {
    pub ident: &'ident Ident,
    pub entries: &'entries [T],
}
impl<T: StringEnum> ToTokens for DisplayStringEnum<'_, '_, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let formatter_ident = format_ident!("formatter");
        let ident = self.ident;
        let entries: Vec<_> = self
            .entries
            .iter()
            .map(|entry| StringEnum::write_str(entry, &formatter_ident))
            .collect();
        let result = quote! {
            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, #formatter_ident: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#entries)*
                    }
                }
            }
        };
        tokens.extend(result);
    }
}
/// Adds TryFrom<&str> and TryFrom<String> for the type
///
/// By passing them to from_str
pub struct OtherTryStringConverts<'ident, 'error> {
    pub ident: &'ident Ident,
    pub error: &'error Ident,
}

impl ToTokens for OtherTryStringConverts<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident;
        let error = self.error;
        let result = quote! {
            #[automatically_derived]
            impl std::convert::TryFrom<&str> for #ident {
                type Error = #error;
                #[inline(always)]
                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    use std::str::FromStr;
                    Self::from_str(value)
                }
            }
            #[automatically_derived]
            impl std::convert::TryFrom<String> for #ident {
                type Error = #error;
                #[inline(always)]
                fn try_from(value: String) -> Result<Self, Self::Error> {
                    use std::str::FromStr;
                    Self::from_str(&value)
                }
            }
        };
        tokens.extend(result);
    }
}

pub trait StringEnum {
    fn variant_name(&self) -> &Ident;

    fn str_value(&self) -> Cow<'_, LitStr>;

    fn as_ref_str(&self) -> TokenStream {
        let variant_name = self.variant_name();
        let str_value = self.str_value();
        quote! {
            Self::#variant_name => #str_value,
        }
    }
    fn write_str(&self, formatter: &Ident) -> TokenStream {
        let variant_name = self.variant_name();
        let str_value = self.str_value();
        quote! {
            Self::#variant_name => #formatter.write_str(#str_value),
        }
    }
    #[allow(clippy::wrong_self_convention)]
    fn from_str_impl(&self) -> TokenStream {
        let variant_name = self.variant_name();
        let str_value = self.str_value();
        quote! {
            #str_value => Ok(Self::#variant_name),
        }
    }
    #[allow(dead_code)]
    #[allow(clippy::wrong_self_convention)]
    fn from_str_no_error(&self) -> TokenStream {
        let variant_name = self.variant_name();
        let str_value = self.str_value();
        quote! {
            #str_value => Self::#variant_name,
        }
    }
}

pub struct AsRefImpl<'ident, 'error, 'entries, T: StringEnum> {
    pub ident: &'ident Ident,
    pub entries: &'entries [T],
    pub error: &'error Ident,
}
impl<T: StringEnum> ToTokens for AsRefImpl<'_, '_, '_, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident;
        let entries = self.entries;
        let error = self.error;
        let as_str = entries
            .iter()
            .map(StringEnum::as_ref_str)
            .collect::<Vec<_>>();
        let from_string = entries
            .iter()
            .map(StringEnum::from_str_impl)
            .collect::<Vec<_>>();
        let other_converts = OtherTryStringConverts { ident, error };
        let result = quote! {
            #[automatically_derived]
            impl std::convert::AsRef<str> for #ident {
                fn as_ref(&self) -> &str {
                    match self {
                        #(#as_str)*
                    }
                }
            }
            impl std::str::FromStr for #ident {
                type Err = #error;
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value {
                        #(#from_string)*
                        _ => Err(#error::from(value.to_owned())),
                    }
                }
            }
            #other_converts
        };
        tokens.extend(result);
    }
}
#[allow(dead_code)]
pub fn display_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
