// SPDX-License-Identifier: (MIT OR Apache-2.0)

use alloc::str;
use core::error::Error;
use core::fmt::{self, Display};
use core::num::ParseIntError;

#[derive(Debug)]
pub enum ISOError<T> {
    Io(T),
    Utf8(str::Utf8Error),
    InvalidFs(&'static str),
    ParseInt(ParseIntError),
    ReadSize(usize, usize),
    Nom(nom::error::ErrorKind),
}

impl<T: core::fmt::Debug> Display for ISOError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ISOError::Io(ref err) => write!(f, "IO error: {:?}", err),
            ISOError::Utf8(ref err) => write!(f, "UTF8 error: {}", err),
            ISOError::InvalidFs(msg) => write!(f, "Invalid ISO9660: {}", msg),
            ISOError::ParseInt(ref err) => write!(f, "Int parse error: {}", err),
            ISOError::ReadSize(size, size_read) => write!(
                f,
                "Reading '{}' bytes block returned '{}' bytes",
                size, size_read
            ),
            ISOError::Nom(ref err) => write!(f, "Parse error: {:?}", err),
        }
    }
}

impl<T: core::error::Error> Error for ISOError<T> {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            ISOError::Io(ref err) => Some(err),
            ISOError::Utf8(ref err) => Some(err),
            ISOError::ParseInt(ref err) => Some(err),
            _ => None,
        }
    }
}

macro_rules! impl_from_error {
    ($t:ty, $e:expr) => {
        impl<T> From<$t> for ISOError<T> {
            fn from(err: $t) -> ISOError<T> {
                $e(err)
            }
        }
    };
}

impl_from_error!(str::Utf8Error, ISOError::Utf8);
impl_from_error!(ParseIntError, ISOError::ParseInt);

impl<T> From<nom::Err<nom::error::Error<&[u8]>>> for ISOError<T> {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> ISOError<T> {
        ISOError::Nom(match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => e.code,
            nom::Err::Incomplete(_) => panic!(), // XXX
        })
    }
}
