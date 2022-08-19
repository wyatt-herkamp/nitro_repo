macro_rules! gen_dynamic_stage {
    ($v:ident,$($name:ident),*) => {
        #[async_trait::async_trait]
        impl<StorageType: Storage> crate::repository::staging::StageHandler<StorageType> for $v<StorageType>{
            fn staging_repository(&self) -> bool {
                match self {
                    $(
                        $v::$name(handler) => handler.staging_repository(),
                    )*
                    _ => false,
                }
            }
    async fn push(
        &self,
        directory: String,
        storages: Arc<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
        user: UserSafeData,
    ) -> Result<(), InternalError>{
        match self {
            $(
                $v::$name(handler) => handler.push(directory, storages, user).await,
            )*
            _ => unsafe{ std::hint::unreachable_unchecked() },
        }
    }
        }
    };
}
pub(crate) use gen_dynamic_stage;
