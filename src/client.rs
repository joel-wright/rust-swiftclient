//use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;
use hyper;
//use hyper::client::request;
use std::error;
use std::fmt;

use auth::{Auth, AuthError}; //, KeystoneAuthV2};

pub struct SwiftClient<T: Auth> {
    auth: T
}

#[derive(Debug)]
pub enum SwiftClientError {
    Http(hyper::error::Error),
    Auth(AuthError),
    // Fail(String)
}

impl fmt::Display for SwiftClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SwiftClientError::Http(ref err) => write!(f, "HTTP error: {}", err),
            SwiftClientError::Auth(ref err) => write!(f, "Auth error: {}", err),
            // SwiftClientError::Fail(ref s) => write!(f, "Fail: {}", s),
        }
    }
}

impl error::Error for SwiftClientError {
    fn description(&self) -> &str {
        match *self {
            SwiftClientError::Http(ref err) => err.description(),
            SwiftClientError::Auth(ref err) => err.description(),
            // SwiftClientError::Fail(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SwiftClientError::Http(ref err) => Some(err),
            SwiftClientError::Auth(ref err) => Some(err),
            // SwiftClientError::Fail(_) => None,
        }
    }
}

impl<T: Auth> SwiftClient<T> {
    pub fn new(auth: T) -> SwiftClient<T> {
        SwiftClient {
            auth: auth
        }
    }

    // Basic Swift Operations
    //pub fn head_account(&self) -> Result<>

    pub fn head(&self, path: &String) -> Result<response::Response, SwiftClientError> {
        let req_builder = try!(
            self.auth.build_request(
                Method::Head, path
            ).map_err(SwiftClientError::Auth)
        );
        let resp = match req_builder.send() {
            Ok(r) => r,
            Err(e) => return Err(SwiftClientError::Http(e))
        };
        // for header in resp.headers.iter() {
        //     println!("XXX {0:?} --- {1:?}", header.name(), header.value_string());
        // }
        Ok(resp)
    }

    // pub fn put<S>(&self, path: &String, source: &S) ->
    //         Result<response::Response, String> where S: Read {
    //     let req_builder = try!(self.auth.build_request(Method::Put, path));
    // }
    //
    // pub fn get(&self, path: &String) -> Result<response::Response, String> {
    //     let req_builder = try!(self.auth.build_request(Method::Get, path));
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

    // Low level swift operations
}
