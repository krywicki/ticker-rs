use std::fmt;
use std::convert;

use hyper;
use serde_json;

#[derive(Debug)]
pub enum ErrorKind {
    HttpError,
    DecodeError,
    IOError,
    Unknown
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            ErrorKind::HttpError => "HTTP Error",
            ErrorKind::DecodeError => "Decode Error",
            ErrorKind::IOError => "IO Error",
            ErrorKind::Unknown => "Unknown Error"
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String
}

impl Error {
    pub fn new<T:AsRef<str>>(kind:ErrorKind, msg:T) -> Error {
        Error {
            kind,
            msg: msg.as_ref().into()
        }
    }

    pub fn simple(kind:ErrorKind) -> Error {
        Error {
            kind,
            msg:String::new()
        }
    }
}

impl convert::From<hyper::Error> for Error {
    fn from(_err: hyper::Error) -> Self {
        let msg:String;

        if _err.is_body_write_aborted() {
            msg = "body write aborted".into();
        } else if _err.is_canceled() {
            msg = "canceled".into();
        } else if _err.is_closed() {
            msg = "closed".into();
        } else if _err.is_connect() {
            msg = "connect failure".into();
        } else if _err.is_incomplete_message() {
            msg = "incomplete message".into();
        } else if _err.is_parse() {
            msg = "parse failure".into();
        } else if _err.is_timeout() {
            msg = "timed out".into();
        } else if _err.is_user() {
            msg = "user error".into();
        } else {
            msg = "unknown hyper error".into()
        }

        Error::new(ErrorKind::HttpError, msg)
    }
}

impl convert::From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::new(ErrorKind::DecodeError, err.to_string())
    }
}

impl convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::IOError, err.to_string())
    }
}

impl std::convert::From<std::sync::mpsc::RecvError> for Error {
    fn from(_err: std::sync::mpsc::RecvError) -> Self {
        Error::new(ErrorKind::Unknown, _err.to_string())
    }
}
