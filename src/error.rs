use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("error parsing tags: {0}")]
    TagError(String),

    #[error("error parsing prefix: {0}")]
    PrefixError(String),

    #[error("error parsing command: {0}")]
    CommandError(String),

    #[error("error parsing params: {0}")]
    ParamsError(String),

    #[error("generic parse error: {0}")]
    GenericError(String),
}
