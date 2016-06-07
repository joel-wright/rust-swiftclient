use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;
use hyper::client::RequestBuilder;

use std::vec::Vec;

use auth::sessions::Auth;
use auth::errors::AuthRequestError;

pub enum Format {
    JSON,
    XML,
    Plain
}

pub trait RunSwiftRequest {
    fn run_request(&self, auth: &Auth)
        -> Result<response::Response, AuthRequestError>;
}

/*
 * Get Account
 */
pub struct GetAccount {
    marker: Option<String>,
    limit: u32,
    prefix: Option<String>,
    end_marker: Option<String>,
    format: Format,
    headers: Headers
}

impl GetAccount {
    pub fn new() -> GetAccount {
        GetAccount{
            marker: None,
            limit: 10000,
            prefix: None,
            end_marker: None,
            format: Format::JSON,
            headers: Headers::new()
        }
    }
}

impl RunSwiftRequest for GetAccount {
    fn run_request(&self, auth: &Auth)
            -> Result<response::Response, AuthRequestError> {
        let mut path = "".to_string();
        let mut query_params = Vec::new();
        match self.marker {
            Some(ref m) => query_params.push(format!("{}={}", "marker", m)),
            None => ()
        };
        match self.prefix {
            Some(ref p) => query_params.push(format!("{}={}", "prefix", p)),
            None => ()
        };
        match self.end_marker {
            Some(ref e) => query_params.push(format!("{}={}", "end_marker", e)),
            None => ()
        }
        query_params.push(format!("{}={}", "limit", self.limit));
        match self.format {
            Format::JSON => query_params.push(format!("{}={}", "format", "json")),
            Format::XML => query_params.push(format!("{}={}", "format", "xml")),
            Format::Plain => ()
        };
        if !query_params.is_empty() {
            path = "?".to_string() + &query_params.join("&").to_string()
        };
        match build_request(auth, Method::Get, path, self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Head Account
 */
pub struct HeadAccount {
    headers: Headers
}

impl HeadAccount {
    pub fn new() -> HeadAccount {
        HeadAccount{
            headers: Headers::new()
        }
    }
}

impl RunSwiftRequest for HeadAccount {
    fn run_request(&self, auth: &Auth)
            -> Result<response::Response, AuthRequestError> {
        let path = "".to_string();
        match build_request(auth, Method::Head, path, self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Post Account
 */
pub struct PostAccount {
    headers: Headers
}

impl PostAccount {
    pub fn new() -> PostAccount {
        PostAccount{
            headers: Headers::new()
        }
    }
}

impl RunSwiftRequest for PostAccount {
    fn run_request(&self, auth: &Auth)
            -> Result<response::Response, AuthRequestError> {
        let path = "".to_string();
        match build_request(auth, Method::Post, path, self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Helper functions
 */
fn build_request(auth: &Auth, method: Method, path: String, headers: Headers)
        -> Result<RequestBuilder, AuthRequestError> {
    let req_builder = try!(
        auth.build_request(
            method, path, headers
        ).map_err(AuthRequestError::Auth)
    );
    Ok(req_builder)
}

fn run_request(request: RequestBuilder)
        -> Result<response::Response, AuthRequestError> {
    let resp = match request.send() {
        Ok(r) => r,
        Err(e) => return Err(AuthRequestError::Http(e))
    };
    Ok(resp)
}
