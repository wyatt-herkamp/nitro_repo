macro_rules! unwrap_or_not_found {
    ($value:expr) => {
        if let Some(value) = $value {
            value
        } else {
            return Ok(actix_web::HttpResponse::NotFound().finish());
        }
    };
}
pub(crate) use unwrap_or_not_found;

macro_rules! get_storage {
    ($storage_handler:ident, $storage_name:ident) => {
        if let Some(value) = $storage_handler
            .get_storage_by_name(&$storage_name)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
        {
            value
        } else {
            return Ok(actix_web::HttpResponse::NotFound().finish().into());
        }
    };
}
macro_rules! get_repository {
    ($storage:ident, $repository:ident) => {
        if let Some(value) = $storage
            .get_repository(&$repository)
            .map_err(actix_web::error::ErrorInternalServerError)?
        {
            value
        } else {
            return Ok(actix_web::HttpResponse::NotFound().finish().into());
        }
    };
}

macro_rules! take_repository {
    ($storage:ident, $repository:ident) => {
        if let Some(repository) = $storage.remove_repository_for_updating($repository) {
            match Removed::try_into(repository) {
                Ok((name, arc)) => match Arc::try_unwrap(arc) {
                    Ok(ok) => (name, ok),
                    Err(v) => {
                        let arc1 = (*v).clone();
                        (name, arc1)
                    }
                },
                Err(v) => {
                    let (name, arc) = (*v).clone();
                    let arc = (*arc).clone();
                    (name, arc)
                }
            }
        } else {
            return Ok(actix_web::HttpResponse::NotFound().finish().into());
        }
    };
}
pub(crate) use get_repository;
pub(crate) use get_storage;
pub(crate) use take_repository;
