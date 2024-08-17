use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Result};
pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, .. } = derive_input;

    let Data::Enum(data_enum) = data else {
        return Err(syn::Error::new(ident.span(), "Expected an enum"));
    };

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
            fn get_storage(&self) -> nr_storage::DynStorage {
                match self {
                    #(
                        #ident::#variants(variant) => variant.get_storage(),
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
            async fn handle_get(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_get(request).await,
                    )*
                }
            }
            async fn handle_post(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_post(request).await,
                    )*
                }
            }
            async fn handle_put(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_put(request).await,
                    )*
                }
            }
            /// Handles a PATCH Request to a Repo
            async fn handle_patch(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_patch(request).await,
                    )*
                }
            }
            async fn handle_delete(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_delete(request).await,
                    )*
                }
            }
            async fn handle_head(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_head(request).await,
                    )*
                }
            }
            async fn handle_other(
                &self,
                request: RepositoryRequest,
            ) -> Result<RepoResponse, RepositoryHandlerError> {
                match self {
                    #(
                        #ident::#variants(variant) => variant.handle_other(request).await,
                    )*
                }
            }
        }
    };
    Ok(result)
}
