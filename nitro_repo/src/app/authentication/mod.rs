//! The tower layer that reads credentials off every request.
//!
//! The authentication types themselves — `AuthenticationRaw`, the extractors, the session store
//! — live in `nr-web-core`, because repository code needs them and cannot depend on the server.
//! What is left here is the middleware that installs them, which is application wiring: it is
//! mounted on the app router and it links spans into the OpenTelemetry context, neither of which
//! belongs below the application.

pub mod layer;
