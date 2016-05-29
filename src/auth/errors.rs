use hyper;
use rustc_serialize;
use std::error;
use std::fmt;

///////////////////////////
// Errors for AuthRequest
///////////////////////////
#[derive(Debug)]
pub enum AuthRequestError {
    Http(hyper::error::Error),
    Auth(AuthError)
}

impl fmt::Display for AuthRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthRequestError::Http(ref err) => write!(f, "AuthRequest HTTP error: {}", err),
            AuthRequestError::Auth(ref err) => write!(f, "AuthRequest Auth error: {}", err)
        }
    }
}

impl error::Error for AuthRequestError {
    fn description(&self) -> &str {
        match *self {
            AuthRequestError::Http(ref err) => err.description(),
            AuthRequestError::Auth(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AuthRequestError::Http(ref err) => Some(err),
            AuthRequestError::Auth(ref err) => Some(err)
        }
    }
}

////////////////////
// Errors for Auth
////////////////////
#[derive(Debug)]
pub enum AuthError {
    Http(hyper::error::Error),
    JsonEncode(rustc_serialize::json::EncoderError),
    JsonDecode(rustc_serialize::json::ParserError),
    JsonContent(String),
    Fail(String)
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthError::Http(ref err) => write!(f, "Auth HTTP error: {}", err),
            AuthError::JsonEncode(ref err) => write!(f, "Auth JSON Encode error: {}", err),
            AuthError::JsonDecode(ref err) => write!(f, "Auth JSON Decode error: {}", err),
            AuthError::JsonContent(ref s) => write!(f, "Auth JSON Content error: {}", s),
            AuthError::Fail(ref s) => write!(f, "Auth Fail: {}", s),
        }
    }
}

impl error::Error for AuthError {
    fn description(&self) -> &str {
        match *self {
            AuthError::Http(ref err) => err.description(),
            AuthError::JsonEncode(ref err) => err.description(),
            AuthError::JsonDecode(ref err) => err.description(),
            AuthError::JsonContent(ref s) => s,
            AuthError::Fail(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AuthError::Http(ref err) => Some(err),
            AuthError::JsonEncode(ref err) => Some(err),
            AuthError::JsonDecode(ref err) => Some(err),
            AuthError::JsonContent(_) => None,
            AuthError::Fail(_) => None,
        }
    }
}
