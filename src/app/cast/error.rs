
#[derive(Debug)]
pub enum CastError {
    CastError(rust_cast::errors::Error),
    IoError(std::io::Error),
    ServerError,
    CasterError(&'static str),
}
impl From<rust_cast::errors::Error> for CastError {
    fn from(err: rust_cast::errors::Error) -> Self {
        CastError::CastError(err)
    }
}
impl From<std::io::Error> for CastError {
    fn from(err: std::io::Error) -> Self {
        CastError::IoError(err)
    }
}