#[cfg(not(feature = "std"))]
pub use embedded_io::*;
#[cfg(feature = "std")]
pub use std::io::*;

#[cfg(feature = "std")]
pub trait ErrorType {
    /// Error type of all the IO operations on this type.
    type Error;
}

/// A helper macro, cleanest way I could find to support std under feature flag
/// with embedded_io's associated Error type.
/// ReaderError!(T) => T::Error in no_std
/// ReaderError!(_) => std::io::Error when (feature = "std")
#[cfg(feature = "std")]
#[macro_export]
macro_rules! ReaderError {
    ($R:ty) => {
        std::io::Error
    };
}
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! ReaderError {
    ($R:ty) => {
        <$R as embedded_io::ErrorType>::Error
    };
}

#[cfg(feature = "std")]
impl<T> ErrorType for T {
    type Error = std::io::Error;
}
