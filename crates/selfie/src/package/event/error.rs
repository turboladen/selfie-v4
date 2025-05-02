// use serde::{Deserialize, Serialize};
//
use crate::{commands::runner::CommandError, package::port::PackageRepoError};

pub trait StreamedError: std::fmt::Debug + Send + Sync + 'static {}

impl StreamedError for PackageRepoError {}
impl StreamedError for CommandError {}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StreamedError<E> {
//     message: String,
//     error_code: StreamedErrorCode,
//     source: Option<E>,
// }
//
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub enum StreamedErrorCode {
//     Package(PackageErrorCode),
//     Command(CommandErrorCode),
// }
//
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub enum PackageErrorCode {
//     PackageRepoError,
//     PackageError,
//     PackageListError,
//     PackageDirectoryNotFound,
//     PackageNotFound,
//     MultiplePackagesFound,
//     IoError,
//     ParseError,
// }
//
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub enum CommandErrorCode {
//     Timeout,
//     IoError,
//     StdoutSpawn,
//     StderrSpawn,
//     Callback,
// }
//
// impl From<PackageRepoError> for StreamedError {
//     fn from(value: PackageRepoError) -> Self {
//         let mut me = Self {
//             message: value.to_string(),
//             error_code: StreamedErrorCode::Package(PackageErrorCode::PackageRepoError),
//             source: None,
//         };
//         match value {
//             PackageRepoError::PackageError(package_error) => {
//                 let e = Self::from(package_error);
//                 me.source = Some(Box::new(e));
//                 me
//             }
//             PackageRepoError::PackageListError(package_list_error) => {
//                 let e = Self::from(package_list_error);
//                 me.source = Some(Box::new(e));
//                 me
//             }
//         }
//     }
// }
//
// impl From<PackageError> for StreamedError {
//     fn from(value: PackageError) -> Self {
//         let mut me = Self {
//             message: value.to_string(),
//             error_code: StreamedErrorCode::Package(PackageErrorCode::PackageRepoError),
//             source: None,
//         };
//
//         match value {
//             PackageError::PackageNotFound(package_not_found) => {
//                 let e = Self::from(package_not_found);
//                 me.source = Some(Box::new(e));
//                 me
//             }
//             _ => todo!(),
//         }
//     }
// }
//
// impl From<PackageListError> for StreamedError {
//     fn from(value: PackageListError) -> Self {
//         todo!()
//     }
// }
//
// impl From<CommandError> for StreamedError {
//     fn from(value: CommandError) -> Self {
//         todo!()
//     }
// }
