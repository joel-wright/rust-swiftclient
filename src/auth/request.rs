use hyper::header::Headers;
use hyper::method::Method;
use hyper::client::response;
use hyper::client::RequestBuilder;

use std::sync::Arc;
use std::vec::Vec;

use auth::sessions::Auth;
use auth::errors::AuthRequestError;

pub enum Format {
    JSON,
    XML,
    Plain
}

unsafe impl<AS: Sized+Auth> Send for SwiftConnection<AS> {}
unsafe impl<AS: Sized+Auth> Sync for SwiftConnection<AS> {}

pub struct SwiftConnection<A> {
    auth: Arc<A>
}

impl<AS: Sized+Auth> SwiftConnection<AS> {
    pub fn new(auth: AS) -> SwiftConnection<AS> {
        SwiftConnection{
            auth: Arc::new(auth)
        }
    }

    pub fn head_account(&self) -> HeadAccount<AS> {
        HeadAccount::new(self.auth.clone())
    }

    pub fn get_account(&self) -> GetAccount<AS> {
        GetAccount::new(self.auth.clone())
    }

    pub fn post_account(&self) -> PostAccount<AS> {
        PostAccount::new(self.auth.clone())
    }

    pub fn get_object(&self, path: String) -> GetObject<AS> {
        GetObject::new(self.auth.clone(), path)
    }

    pub fn put_object(&self, path: String, data: Vec<u8>) -> PutObject<AS> {
        PutObject::new(self.auth.clone(), path, data)
    }
}

pub trait RunSwiftRequest {
    fn run_request(&self)
        -> Result<response::Response, AuthRequestError>;
}

/*
 * Get Account
 */
pub struct GetAccount<A> {
    marker: Option<String>,
    limit: u32,
    prefix: Option<String>,
    end_marker: Option<String>,
    format: Format,
    headers: Headers,
    auth: Arc<A>
}

impl<AS: Sized+Auth> GetAccount<AS> {
    pub fn new(auth: Arc<AS>) -> GetAccount<AS> {
        GetAccount {
            marker: None,
            limit: 10000,
            prefix: None,
            end_marker: None,
            format: Format::JSON,
            headers: Headers::new(),
            auth: auth
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for GetAccount<AS> {
    fn run_request(&self)
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
        match build_request(self.auth.as_ref(), Method::Get, path, self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Head Account
 */
pub struct HeadAccount<A> {
    headers: Headers,
    auth: Arc<A>
}

impl<AS: Sized+Auth> HeadAccount<AS> {
    pub fn new(auth: Arc<AS>) -> HeadAccount<AS> {
        HeadAccount {
            headers: Headers::new(),
            auth: auth
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for HeadAccount<AS> {
    fn run_request(&self)
            -> Result<response::Response, AuthRequestError> {
        let path = "".to_string();
        match build_request(
                self.auth.as_ref(),
                Method::Head,
                path, self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Post Account
 */
pub struct PostAccount<A> {
    headers: Headers,
    auth: Arc<A>
}

impl<AS: Sized+Auth> PostAccount<AS> {
    pub fn new(auth: Arc<AS>) -> PostAccount<AS> {
        PostAccount {
            headers: Headers::new(),
            auth: auth
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for PostAccount<AS> {
    fn run_request(&self)
            -> Result<response::Response, AuthRequestError> {
        let path = "".to_string();
        match build_request(
                self.auth.as_ref(),
                Method::Post,
                path, self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}


/*
 * Get Object
 */
pub struct GetObject<A> {
    path: String,
    headers: Headers,
    auth: Arc<A>
}


impl<AS: Sized+Auth> GetObject<AS> {
    pub fn new(auth: Arc<AS>, path: String) -> GetObject<AS> {
        GetObject {
            path: path,
            headers: Headers::new(),
            auth: auth
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for GetObject<AS> {
    fn run_request(&self)
            -> Result<response::Response, AuthRequestError> {
        match build_request(self.auth.as_ref(), Method::Get, self.path.clone(), self.headers.clone()) {
            Ok(req) => run_request(req),
            Err(e) => Err(e)
        }
    }
}


/*
 * Put Object
 */
pub struct PutObject<A> {
    path: String,
    headers: Headers,
    auth: Arc<A>,
    data: Vec<u8>,
}


impl<AS: Sized+Auth> PutObject<AS> {
    pub fn new(auth: Arc<AS>, path: String, data: Vec<u8>) -> PutObject<AS> {
        PutObject {
            path: path,
            headers: Headers::new(),
            auth: auth,
            data: data,
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for PutObject<AS> {
    fn run_request(&self)
            -> Result<response::Response, AuthRequestError> {
        match build_request(self.auth.as_ref(), Method::Put, self.path.clone(), self.headers.clone()) {
            Ok(req) => run_request(req.body(self.data.as_slice())),
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
