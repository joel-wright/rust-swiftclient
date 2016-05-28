use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;

use std::vec::Vec;

use auth::sessions::Auth;
use auth::errors::AuthRequestError;

pub enum Format {
    JSON,
    XML,
    Plain
}

pub struct AuthRequest<T: Auth> {
    auth: T
}

impl<T: Auth> AuthRequest<T> {
    pub fn new(auth: T) -> AuthRequest<T> {
        AuthRequest {
            auth: auth
        }
    }

    // Basic Authenticated Swift Operations
    pub fn get_account(&self, marker: Option<String>, limit: Option<u32>,
                prefix: Option<String>, end_marker: Option<String>, format: Option<Format>,
                headers: Option<Headers>)
            -> Result<response::Response, AuthRequestError> {
        let mut query_params = Vec::new();
        match marker {
            Some(m) => query_params.push(format!("{}={}", "marker", m)),
            None => ()
        };
        match limit {
            Some(l) => query_params.push(format!("{}={}", "limit", l)),
            None => ()
        };
        match prefix {
            Some(p) => query_params.push(format!("{}={}", "prefix", p)),
            None => ()
        };
        match end_marker {
            Some(em) => query_params.push(format!("{}={}", "end_marker", em)),
            None => ()
        };
        match format {
            Some(m) => match m {
                Format::JSON => query_params.push(format!("{}={}", "format", "json")),
                Format::XML => query_params.push(format!("{}={}", "format", "xml")),
                Format::Plain => ()
            },
            None => query_params.push(format!("{}={}", "format", "json"))
        };
        let path = if !query_params.is_empty() {
            "?".to_string() + &query_params.join("&").to_string()
        } else {
            "".to_string()
        };
        let request_headers = match headers {
            Some(h) => h,
            None => Headers::new()
        };
        return self.make_request(Method::Get, &path, request_headers)
    }

    fn make_request(&self, method: Method, path: &String, headers: Headers)
            -> Result<response::Response, AuthRequestError> {
        let req_builder = try!(
            self.auth.build_request(
                method, path, headers
            ).map_err(AuthRequestError::Auth)
        );
        let resp = match req_builder.send() {
            Ok(r) => r,
            Err(e) => return Err(AuthRequestError::Http(e))
        };
        Ok(resp)
    }
}
