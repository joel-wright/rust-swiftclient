//use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;
use hyper;
//use hyper::client::request;
use std::error;
use std::fmt;

use swift_auth::auth_sessions::{Auth, AuthError}; //, KeystoneAuthV2};

pub struct AuthRequest<T: Auth> {
    auth: T
}

#[derive(Debug)]
pub enum AuthRequestError {
    Http(hyper::error::Error),
    Auth(AuthError),
    // Fail(String)
}

impl fmt::Display for AuthRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthRequestError::Http(ref err) => write!(f, "AuthRequest HTTP error: {}", err),
            AuthRequestError::Auth(ref err) => write!(f, "AuthRequest Auth error: {}", err),
            // AuthRequestError::Fail(ref s) => write!(f, "Fail: {}", s),
        }
    }
}

impl error::Error for AuthRequestError {
    fn description(&self) -> &str {
        match *self {
            AuthRequestError::Http(ref err) => err.description(),
            AuthRequestError::Auth(ref err) => err.description(),
            // AuthRequestError::Fail(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AuthRequestError::Http(ref err) => Some(err),
            AuthRequestError::Auth(ref err) => Some(err),
            // AuthRequestError::Fail(_) => None,
        }
    }
}

impl<T: Auth> AuthRequest<T> {
    pub fn new(auth: T) -> AuthRequest<T> {
        AuthRequest {
            auth: auth
        }
    }

    // Basic Authenticated HTTP Operations
    pub fn head(&self, path: &String)
            -> Result<response::Response, AuthRequestError> {
        return self.make_request(Method::Head, path)
    }

    pub fn get(&self, path: &String)
            -> Result<response::Response, AuthRequestError> {
        return self.make_request(Method::Get, path)
    }

    // pub fn put<S>(&self, path: &String, source: &S) ->
    //         Result<response::Response, String> where S: Read {
    //     let req_builder = try!(self.auth.build_request(Method::Put, path));
    // }
    //
    // pub fn post(&self, path: &String, headers??) ->
    //         Result<response::Response, String> {
    //     let req_builder = try!(self.auth.build_request(Method::Post, path));
    // }
    //
    // pub fn delete(&self, path: &String) ->
    //         Result<response::Response, String> {
    //     let req_builder = try!(self.auth.build_request(Method::Delete, path));
    // }

    fn make_request(&self, method: Method, path: &String)
            -> Result<response::Response, AuthRequestError> {
        let req_builder = try!(
            self.auth.build_request(
                method, path
            ).map_err(AuthRequestError::Auth)
        );
        let resp = match req_builder.send() {
            Ok(r) => r,
            Err(e) => return Err(AuthRequestError::Http(e))
        };
        Ok(resp)
    }
}
