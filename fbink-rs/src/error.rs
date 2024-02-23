use thiserror::Error;

#[derive(Error, Debug)]
pub enum FbInkError {
    #[error("FBInk returned EXIT_FAILURE during {0}")]
    ExitFailure(String),
    #[error("{0}")]
    NotSupported(String),
    #[error("Invalid argument provided ({0})")]
    InvalidArgument(String),
    // NoDevice,
    // NoData,
    // TimerExpired,
    // InvalidSequence,
    #[error("{0}")]
    OutOfRange(String),
    // NoSpace,
    // NotImplemented,
    #[error("FBInk failed with error code {0}")]
    Other(i32),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::error::ImageError),
    #[error(transparent)]
    NulStringError(#[from] std::ffi::NulError),
    #[error("Failed to dump the working buffer")]
    SunxiDumpError,
}
