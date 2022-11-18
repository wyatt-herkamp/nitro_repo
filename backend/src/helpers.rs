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
            return Err(
                crate::error::simple_response::SimpleResponse::BadStorageName($storage_name.into())
                    .into(),
            );
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
            return Err(
                crate::error::simple_response::SimpleResponse::BadRepositoryName(
                    $repository.into(),
                )
                .into(),
            );
        }
    };
}

macro_rules! take_repository {
    ($storage:ident, $repository:ident) => {
        if let Some(repository) = $storage.remove_repository_for_updating($repository) {
            match lockfree::map::Removed::try_into(repository) {
                Ok((name, arc)) => match std::sync::Arc::try_unwrap(arc) {
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
macro_rules! read_check_web {
    ($auth:ident, $conn:expr, $config:expr) => {
        if $config.visibility == crate::repository::settings::Visibility::Private {
            let caller = $auth.get_user($conn).await??;
            if let Some(value) = caller.can_read_from(&$config)? {
                return Err(value.into());
            }
        }
    };
}
macro_rules! read_check {
    ($auth:ident, $conn:expr, $config:expr) => {
        if ($config).visibility == crate::repository::settings::Visibility::Private {
            let caller = $auth.get_user($conn).await?;
            match caller {
                Ok(caller) => {
                    if let Some(value) = caller.can_read_from(&$config)? {
                        return Err(value.into());
                    }
                }
                Err(_) => {
                    return Ok(actix_web::HttpResponse::Unauthorized()
                        .append_header(("WWW-Authenticate", "Basic"))
                        .finish()
                        .into());
                }
            }
        }
    };
}
macro_rules! write_check {
    ($auth:ident, $conn:ident, $config:expr) => {{
        if $config.require_token_over_basic {
            if let crate::authentication::Authentication::AuthToken(_, caller) = $auth {
                caller
            } else {
                return Ok(actix_web::HttpResponse::Unauthorized().finish().into());
            }
        } else {
            let caller = $auth.get_user($conn).await??;
            if let Some(value) = caller.can_deploy_to(&$config)? {
                return Err(value.into());
            }
            caller
        }
    }};
}
pub(crate) use get_repository;
pub(crate) use get_storage;
pub(crate) use read_check;
pub(crate) use read_check_web;
pub(crate) use take_repository;
pub(crate) use write_check;
