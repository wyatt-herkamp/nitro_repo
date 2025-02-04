use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, Result,
    parse::{Parse, ParseStream},
};
mod keywords {
    use syn::custom_keyword;
    custom_keyword!(error);
}
pub struct ContainerAttributes {
    pub error: syn::Type,
}
impl Parse for ContainerAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut error: Option<syn::Type> = None;
        while !input.is_empty() {
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::error) {
                let _ = input.parse::<keywords::error>()?;
                let _: syn::Token![=] = input.parse()?;
                error = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }
        let attr = Self {
            error: error.ok_or_else(|| syn::Error::new(input.span(), "Missing error opt"))?,
        };
        Ok(attr)
    }
}
pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        ident, data, attrs, ..
    } = derive_input;

    let Data::Enum(data_enum) = data else {
        return Err(syn::Error::new(ident.span(), "Expected an enum"));
    };
    let ContainerAttributes { error } = attrs
        .iter()
        .find(|v: &&syn::Attribute| v.path().is_ident("repository_handler"))
        .map(|v| v.parse_args::<ContainerAttributes>())
        .transpose()?
        .ok_or_else(|| syn::Error::new(ident.span(), "Missing #[repository_handler]"))?;
    let mut impl_from = Vec::new();
    let mut variants: Vec<_> = Vec::new();
    for variant in data_enum.variants {
        let variant_ident = variant.ident.clone();
        let fields = variant.fields;
        let Fields::Unnamed(fields) = fields else {
            return Err(syn::Error::new(ident.span(), "Expected tuple variant"));
        };
        if fields.unnamed.len() != 1 {
            return Err(syn::Error::new(
                ident.span(),
                "Expected tuple variant with one field",
            ));
        }
        let field = fields.unnamed.first().unwrap();
        let ty = &field.ty;
        let from = quote! {
            impl std::convert::From<#ty> for #ident {
                fn from(value: #ty) -> Self {
                    #ident::#variant_ident(value)
                }
            }
        };
        impl_from.push(from);
        variants.push(variant_ident);
    }

    let result = quote! {
        #(
            #impl_from
        )*

        impl Repository for #ident {
            type Error = #error;
            fn get_storage(&self) -> nr_storage::DynStorage {
                match self {
                    #(
                        #ident::#variants(variant) => variant.get_storage(),
                    )*
                }
            }
            fn site(&self) -> NitroRepo{
                match self {
                    #(
                        #ident::#variants(variant) => variant.site(),
                    )*
                }
            }
            fn name(&self) -> String{
                match self {
                    #(
                        #ident::#variants(variant) => variant.name(),
                    )*
                }
            }
            fn id(&self) -> uuid::Uuid{
                match self {
                    #(
                        #ident::#variants(variant) => variant.id(),
                    )*
                }
            }
            fn visibility(&self) -> Visibility{
                match self {
                    #(
                        #ident::#variants(variant) => variant.visibility(),
                    )*
                }
            }
            fn is_active(&self) -> bool{
                match self {
                    #(
                        #ident::#variants(variant) => variant.is_active(),
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
            fn full_type(&self) -> &'static str {
                match self {
                    #(
                        #ident::#variants(variant) => variant.full_type(),
                    )*
                }
            }

            fn config_types(&self) -> Vec<&str> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.config_types(),
                    )*
                }
            }
            async fn reload(&self) -> Result<(), RepositoryFactoryError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.reload().await,
                    )*
                }
            }
            async fn resolve_project_and_version_for_path(
                &self,
                path: &StoragePath,
            ) -> Result<ProjectResolution, Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.resolve_project_and_version_for_path(path).await.map_err(Self::Error::from),
                    )*
                }
            }
            async fn handle_get(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_get(request).await.map_err(Self::Error::from),
                    )*
                }
            }
            async fn handle_post(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_post(request).await.map_err(Self::Error::from),
                    )*
                }
            }
            async fn handle_put(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_put(request).await.map_err(Self::Error::from),
                    )*
                }
            }
            /// Handles a PATCH Request to a Repo
            async fn handle_patch(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_patch(request).await.map_err(Self::Error::from),
                    )*
                }
            }
            async fn handle_delete(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_delete(request).await.map_err(Self::Error::from),
                    )*
                }
            }
            async fn handle_head(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_head(request).await.map_err(Self::Error::from),
                    )*
                }
            }
            async fn handle_other(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse,  Self::Error> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_other(request).await.map_err(Self::Error::from),
                    )*
                }
            }
        }
    };
    Ok(result)
}
