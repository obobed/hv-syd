#[derive(Debug)]
enum Error {
    NotFound,
}

impl warp::reject::Reject for Error {}
impl core::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => write!(f, "not found"),
        }
    }
}
