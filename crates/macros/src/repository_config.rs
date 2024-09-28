use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Result;
use syn::{DeriveInput, Ident, LitStr};
mod keywords {
    use syn::custom_keyword;
    custom_keyword!(name);
}
#[derive(Debug)]
pub struct ContainerAttrs {
    pub config_name: LitStr,
}

impl Parse for ContainerAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<LitStr> = None;
        while !input.is_empty() {
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::name) {
                let _ = input.parse::<keywords::name>()?;
                let _: syn::Token![=] = input.parse()?;
                name = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }
        let attr = Self {
            config_name: name.ok_or_else(|| syn::Error::new(input.span(), "Missing name opt"))?,
        };
        Ok(attr)
    }
}

pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let container_attr = derive_input
        .attrs
        .iter()
        .find(|v| v.path().is_ident("repository_config"))
        .map(|v| v.parse_args::<ContainerAttrs>())
        .transpose()?
        .ok_or_else(|| {
            syn::Error::new(derive_input.ident.span(), "Missing #[repository_config]")
        })?;
    let ContainerAttrs { config_name } = container_attr;
    let config = derive_input.ident.clone();
    let result = quote! {
        pub async fn get_config(
            storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
            database: actix_web::web::Data<sea_orm::DatabaseConnection>,
            auth: crate::authentication::Authentication,
            path_params: actix_web::web::Path<(String, String)>,
        ) -> actix_web::Result<actix_web::HttpResponse> {
            use crate::storage::models::Storage;
            use crate::system::permissions_checker::CanIDo;
            let user = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let repository = crate::helpers::get_repository!(storage, repository_name);
            if let  crate::repository::handler::DynamicRepositoryHandler::Maven( repository) = repository.as_ref() {
                if let crate::repository::maven::MavenHandler::$maven_type(ref repository) = repository {
                    let value = crate::repository::settings::RepositoryConfigHandler::<#config>::get(repository);
                    return Ok(actix_web::HttpResponse::Ok().json(value));
                }
            }
            return Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()));
        }
        pub async fn set_config(
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
                database: actix_web::web::Data<sea_orm::DatabaseConnection>,
                auth: crate::authentication::Authentication,
                path_params: actix_web::web::Path<(String, String)>,
                body: actix_web::web::Json<$config>,
            ) -> actix_web::Result<actix_web::HttpResponse> {
                use crate::storage::models::Storage;
                use crate::repository::handler::Repository;
                use crate::system::permissions_checker::CanIDo;
                let user = auth.get_user(&database).await??;
                user.can_i_edit_repos()?;
                let (storage_name, repository_name) = path_params.into_inner();
                let storage = crate::helpers::get_storage!(storage_handler, storage_name);
                let  (name,mut repository) = crate::helpers::take_repository!(storage, repository_name);
                let body = body.into_inner();

                let result = if let  crate::repository::handler::DynamicRepositoryHandler::Maven(ref mut repository) = repository {
                    if let crate::repository::maven::MavenHandler::$maven_type(ref mut repository) = repository {
                        let _value = crate::repository::settings::RepositoryConfigHandler::<$config>::get(repository);
                        let value = crate::repository::settings::RepositoryConfigHandler::<$config>::update( repository, body).map(|_| true);
                        if let Err(e) = storage.save_repository_config(repository.get_repository(),  crate::repository::settings::RepositoryConfigHandler::<#config>::get(repository)).await{
                            tracing::error!("{}", e);
                        }
                        value
                    }else{
                        Ok(false)
                    }
                }else {
                    Ok(false)
                };
                storage.add_repository_for_updating(name, repository,false).await.expect("Failed to add repository for updating");
                if result?{
                    Ok(actix_web::HttpResponse::NoContent().finish())
                }else{
                    Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()))
                }
        }
        pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
                cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", #config_name)])
                .route(actix_web::web::get().to(get_config))
                .route(actix_web::web::put().to(set_config)));
        }
    };
    let module_name: Ident = Ident::new(
        &format!("web_{}", config_name.value()),
        derive_input.ident.span(),
    );
    let wrapped = quote! {
        pub mod #module_name {
            #result
        }
    };
    Ok(wrapped)
}
