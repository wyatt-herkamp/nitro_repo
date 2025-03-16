pub mod request_id;
pub mod request_span;
pub trait HttpTraceValue {
    fn value(&self) -> impl tracing::Value;
}

impl HttpTraceValue for http::Method {
    fn value(&self) -> impl tracing::Value {
        tracing::field::display(self)
    }
}

impl HttpTraceValue for http::Version {
    fn value(&self) -> impl tracing::Value {
        match self {
            &http::Version::HTTP_09 => "0.9".to_owned(),
            &http::Version::HTTP_10 => "1.0".to_owned(),
            &http::Version::HTTP_11 => "1.1".to_owned(),
            &http::Version::HTTP_2 => "2".to_owned(),
            &http::Version::HTTP_3 => "3".to_owned(),
            other => format!("{:?}", other)
                .trim_start_matches("HTTP/")
                .to_owned(),
        }
    }
}
impl HttpTraceValue for http::StatusCode {
    fn value(&self) -> impl tracing::Value {
        self.as_u16() as u64
    }
}
