use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Result, Type};
pub struct StorageConfigVariant {
    pub name: Ident,
    pub ty: Type,
}
impl StorageConfigVariant {
    pub fn impl_from(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        quote! {
            #[automatically_derived]
            impl From<#ty> for StorageConfig{
                fn from(v: #ty) -> Self{
                    StorageConfig::#name(v)
                }
            }
        }
    }
}
impl ToTokens for StorageConfigVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote! {
            #name(#ty)
        });
    }
}

pub struct DynamicStorageVariant {
    pub name: Ident,
    pub ty: Type,
}
impl DynamicStorageVariant {
    pub fn impl_from(&self, type_name: &Ident) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        quote! {
            #[automatically_derived]
            impl From<#ty> for #type_name{
                fn from(v: #ty) -> Self{
                    Self::#name(v)
                }
            }
        }
    }
}

impl ToTokens for DynamicStorageVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote! {
            #name(#ty)
        });
    }
}
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        ident: name, data, ..
    } = input;

    let Data::Enum(enum_type) = data else {
        return Err(syn::Error::new(name.span(), "Expected an enum"));
    };
    let mut config_variants = vec![];
    let mut dynamic_variants = vec![];
    let mut storage_types = vec![];
    for variant in enum_type.variants {
        let name = variant.ident;
        let fields = variant.fields;
        if fields.len() != 2 {
            return Err(syn::Error::new(
                name.span(),
                "Expected 2 fields in the enum",
            ));
        }
        let mut fields = fields.into_iter();
        let dynamic_storage_type = fields.next().unwrap();
        let config_storage_type = fields.next().unwrap();
        config_variants.push(StorageConfigVariant {
            name: name.clone(),
            ty: config_storage_type.ty,
        });
        dynamic_variants.push(DynamicStorageVariant {
            name: name.clone(),
            ty: dynamic_storage_type.ty,
        });
        storage_types.push(name);
    }

    let final_enums = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum StorageTypes{
            #(#storage_types,)*
        }
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "storage_type", content = "config")]
        pub enum StorageConfig {
            #(#config_variants,)*
        }
        #[derive(Debug)]
        pub enum DynamicStorage{
            BadStorage(BadStorage),
            #(#dynamic_variants,)*
        }
    };
    let impl_from_config = config_variants.iter().map(|v| v.impl_from());
    let impl_from_dynamic = dynamic_variants.iter().map(|v| v.impl_from(&name));
    let implementations = quote! {
        #(#impl_from_config)*
        #(#impl_from_dynamic)*
         #[automatically_derived]
         impl Storage for #name {
             type Repository = DynamicRepositoryHandler<#name>;
             async fn create_new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)>
                 where Self: Sized{
                     todo!()
             }
             async fn new(config: StorageSaver) -> Result<Self, (StorageError, StorageSaver)>
                     where Self: Sized{
                     todo!()
             }
             async fn get_repos_to_load(&self) -> Result<HashMap<String, RepositoryConfig>, StorageError>{
                     todo!()
             }

             fn add_repo_loaded<R: Into<Self::Repository> + Send>(&self, repo: R) -> Result<(), StorageError>{
                 todo!()
    }
             fn unload(&mut self) -> Result<(), StorageError>{
                     todo!()
             }

            fn storage_config(&self) -> &StorageSaver{
                     todo!()
             }

            async fn create_repository<R: Into<Self::Repository> + Send>(&self,repository: R) -> Result<Arc<Self::Repository>, StorageError>{
                todo!()
             }
            async fn delete_repository<S: AsRef<str> + Send>(&self,repository: S,delete_files: bool) -> Result<(), StorageError>{
                     todo!()
            }
            fn get_repository_list(&self) -> Result<Vec<RepositoryConfig>, StorageError>{
                     todo!()
             }
            fn get_repository<S: AsRef<str>>(&self,repository: S) -> Result<Option<Arc<Self::Repository>>, StorageError>{
                     todo!()
            }
            fn remove_repository_for_updating(&self,repository: &str,) -> Option<Removed<String, Arc<Self::Repository>>>{
                     todo!()
             }
            /// Will update all configs for the Repository
            async fn add_repository_for_updating(&self,name: String, repository_arc: Self::Repository, save: bool, ) -> Result<(), StorageError>{
                     todo!()
            }
            async fn save_file(&self,repository: &RepositoryConfig,file: &[u8],location: &str,) -> Result<bool, StorageError>{
                todo!()
             }
            fn write_file_stream<S: Stream<Item = Bytes> + Unpin + Send + Sync + 'static>(&self,repository: &RepositoryConfig,s: S,location: &str,) -> Result<bool, StorageError>{
                     todo!()
             }
     async fn delete_file(
         &self,
         repository: &RepositoryConfig,
         location: &str,
     ) -> Result<(), StorageError>{
                     todo!()
             }
     /// Gets tje File as a StorageFileResponse
     /// Can be converted for Web Responses
     async fn get_file_as_response(
         &self,
         repository: &RepositoryConfig,
         location: &str,
     ) -> Result<StorageFileResponse, StorageError>{
                     todo!()
             }
     /// Returns Information about the file
     async fn get_file_information(
         &self,
         repository: &RepositoryConfig,
         location: &str,
     ) -> Result<Option<StorageFile>, StorageError>{
                     todo!()
             }
     /// Gets the File as an Array of Bytes
     /// Used for internal processing
     async fn get_file(
         &self,
         repository: &RepositoryConfig,
         location: &str,
     ) -> Result<Option<Vec<u8>>, StorageError>{
                     todo!()
             }
     /// Gets a Repository Config
     async fn get_repository_config<ConfigType: DeserializeOwned>(
         &self,
         repository: &RepositoryConfig,
         config_name: &str,
     ) -> Result<Option<ConfigType>, StorageError>{
                     todo!()
             }

     async fn save_repository_config<ConfigType: RepositoryConfigType>(
         &self,
         repository: &RepositoryConfig,
         config: &ConfigType,
     ) -> Result<(), StorageError>{
                     todo!()
             }

     async fn list_files<SP: Into<StoragePath> + Send>(
         &self,
         repository: &str,
         path: SP,
     ) -> Result<Vec<SystemStorageFile>, StorageError>{
                     todo!()
             }
         }
     };
    let result = quote! {
        #final_enums

        const _: () = {
            extern crate serde as _serde;
            #implementations
        };

    };
    Ok(result)
}
