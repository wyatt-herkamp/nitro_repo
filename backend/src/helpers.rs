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
