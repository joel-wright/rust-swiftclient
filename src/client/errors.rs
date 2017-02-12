use reqwest;
use std::error;
use std::fmt;

use auth::errors::AuthError;

/*
 * Errors for Swift client
 */

#[derive(Debug)]
pub enum SwiftError {
    Http(reqwest::Error),
    Auth(AuthError)
    // there will probably be others
}

impl fmt::Display for SwiftError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SwiftError::Http(ref err) => write!(
                f, "Swift request HTTP error: {}", err),
            SwiftError::Auth(ref err) => write!(
                f, "Swift request Auth error: {}", err)
        }
    }
}

impl error::Error for SwiftError {
    fn description(&self) -> &str {
        match *self {
            SwiftError::Http(ref err) => err.description(),
            SwiftError::Auth(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SwiftError::Http(ref err) => Some(err),
            SwiftError::Auth(ref err) => Some(err)
        }
    }
}
