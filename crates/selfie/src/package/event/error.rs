use thiserror::Error;

use crate::{commands::runner::CommandError, package::port::PackageRepoError};

#[derive(Debug, Error, Clone)]
pub enum StreamedError {
    #[error(transparent)]
    PackageRepoError(#[from] PackageRepoError),
    #[error(transparent)]
    CommandError(#[from] CommandError),
}
