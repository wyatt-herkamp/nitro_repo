//!
//! ```rust,ignore
//! #[derive(Scopes)]
//! pub enum AuthTokenScopes{
//!    /// ReadRepository allows the user to read from the repository
//!    #[scope(title = "Read Repository", parent = "Repository")]
//!    ReadRepository,
//!    /// WriteRepository allows the user to write to the repository
//!    #[scope(title = "Write Repository", parent = "Repository")]
//!    WriteRepository,
//!   /// EditRepository allows the user to edit the repository
//!   #[scope(title = "Edit Repository", parent = "Repository")]
//!  EditRepository,
//! }
//! ```
use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::Ident;
use syn::Lit;
use syn::LitStr;
use syn::Result;
mod keywords {
    syn::custom_keyword!(title);
    syn::custom_keyword!(parent);
}
#[derive(Debug)]
pub struct ScopeAttribute {
    title: LitStr,
    parent: Option<LitStr>,
}
pub struct ScopeEntry {
    ident: Ident,
    attribute: ScopeAttribute,
    docs: LitStr,
}
impl ScopeEntry {
    pub fn description_tokens(&self) -> TokenStream {
        let Self {
            ident,
            attribute,
            docs,
        } = self;
        let ScopeAttribute { title, parent } = attribute;
        let parent = if let Some(parent) = parent {
            quote! { Some(#parent) }
        } else {
            quote! { None }
        };
        let scope = quote! {
            Self::#ident => ScopeDescription{
                key: Self::#ident,
                description: #docs,
                name: #title,
                parent: #parent,
                ..std::default::Default::default()
            },
        };
        scope
    }
    pub fn as_str(&self) -> TokenStream {
        let name_as_string = self.name_as_string();
        let ident = &self.ident;
        let scope = quote! {
            Self::#ident => #name_as_string,
        };
        scope
    }

    pub fn from_string(&self) -> TokenStream {
        let name_as_string = self.name_as_string();
        let ident = &self.ident;
        let scope = quote! {
            #name_as_string => Ok(Self::#ident),
        };
        scope
    }
    fn name_as_string(&self) -> String {
        let Self { ident, .. } = self;
        ident.to_string()
    }
}
impl syn::parse::Parse for ScopeAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut title = None;
        let mut parent = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::title) {
                input.parse::<keywords::title>()?;
                input.parse::<syn::Token![=]>()?;
                title = input.parse()?;
            } else if lookahead.peek(keywords::parent) {
                input.parse::<keywords::parent>()?;
                input.parse::<syn::Token![=]>()?;
                parent = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        let title = title.ok_or_else(|| input.error("No title found"))?;
        Ok(Self { title, parent })
    }
}

pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, .. } = derive_input;
    let Data::Enum(data_enum) = data else {
        return Err(syn::Error::new(ident.span(), "Expected an enum"));
    };
    let mut entries = Vec::new();
    for variant in data_enum.variants {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new_spanned(variant, "Expected a unit variant"));
        }

        let mut attribute = None;
        let mut doc_comments = String::new();
        for attr in variant.attrs.iter() {
            if attr.path().is_ident("doc") {
                let doc_str = doc_attr_to_string(attr)?;
                doc_comments.push_str(doc_str.trim());
            } else if attr.path().is_ident("scope") {
                let meta = attr.parse_args::<ScopeAttribute>()?;
                attribute = Some(meta);
            }
        }
        let attribute = attribute
            .ok_or_else(|| syn::Error::new_spanned(&variant, "Expected a scope attribute"))?;
        let doc_comment = LitStr::new(&doc_comments, variant.ident.span());
        entries.push(ScopeEntry {
            ident: variant.ident,
            attribute,
            docs: doc_comment,
        });
    }
    let descriptions = entries.iter().map(ScopeEntry::description_tokens);
    let as_str = entries.iter().map(ScopeEntry::as_str);
    let from_string = entries.iter().map(ScopeEntry::from_string);
    let result = quote! {
        impl #ident {
            pub fn description(&self) -> ScopeDescription {
                match self {
                    #(#descriptions)*
                }
            }
        }
        impl std::fmt::Display for #ident{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = self.as_ref();
                write!(f, "{}", name)
            }
        }
        impl std::convert::AsRef <str> for #ident{
            fn as_ref(&self) -> &str {
                match self {
                    #(#as_str)*
                }
            }
        }
        impl std::convert::TryFrom<&str> for #ident{
            type Error = InvalidScope;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    #(#from_string)*
                    _ => Err(InvalidScope::from(value.to_owned())),
                }
            }
        }
        impl  std::str::FromStr for #ident{
            type Err = InvalidScope;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }
        impl std::convert::TryFrom<String> for #ident{
            type Error = InvalidScope;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }
        const _: () ={
            impl serde::Serialize for #ident{
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let name: &str = self.as_ref();
                    serializer.serialize_str(name)
                }
            }
            impl<'de> serde::Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let value = String::deserialize(deserializer)?;
                    Self::try_from(value).map_err(serde::de::Error::custom)
                }
            }
        };
    };
    Ok(result)
}

fn doc_attr_to_string(attr: &Attribute) -> Result<String> {
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
