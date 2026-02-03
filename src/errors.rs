/// Common error type for all possible errors. Payload is a string with a description giving
/// context to the error.
#[derive(Debug)]
pub enum Error {
    /// HTTP client error.
    Http(&'static str),
    /// JSON parse error.
    ParseJson(&'static str),
    /// ICS event parse error.
    ParseEvent(&'static str),
    /// UTF-8 conversion error.
    ParseUtf8,
    /// Generic date/time error.
    DateTime(&'static str),
}
