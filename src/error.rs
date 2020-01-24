use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ParseError {
    #[error("error parsing tags: {0}")]
    TagError(nom::Err<nom::error::ErrorKind>),

    #[error("error parsing prefix: {0}")]
    PrefixError(nom::Err<nom::error::ErrorKind>),

    #[error("error parsing tags: {0}")]
    CommandError(nom::Err<nom::error::ErrorKind>),

    #[error("error parsing params: {0}")]
    ParamsError(nom::Err<nom::error::ErrorKind>),
}
