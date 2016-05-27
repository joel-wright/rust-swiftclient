//use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;

use auth::sessions::Auth; //, KeystoneAuthV2};
use auth::errors::AuthRequestError;

pub struct AuthRequest<T: Auth> {
    auth: T
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
