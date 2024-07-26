pub fn bad_request(message: &str) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(message.to_owned())
}
