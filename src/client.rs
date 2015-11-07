//use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;
//use hyper::client::request;

use auth::Auth; //, KeystoneAuthV2};

pub struct SwiftClient<T: Auth> {
    auth: T
}

impl<T: Auth> SwiftClient<T> {
    pub fn new(auth: T) -> SwiftClient<T> {
        SwiftClient {
            auth: auth
        }
    }

    pub fn head(&mut self, path: &String) -> Result<response::Response, String> {
        let req_builder = try!(self.auth.build_request(Method::Head, path));
        let resp = match req_builder.send() {
            Ok(r) => r,
            Err(_) => return Err(String::from("Request Failed"))
        };
        Ok(resp)
    }
}
