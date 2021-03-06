use hyper::header::Headers;
use hyper::method::Method;
use reqwest::{Body, RequestBuilder, Response};

use std::fmt::Display;
use std::sync::Arc;
use std::vec::Vec;

//use auth::errors::AuthError;
use auth::sessions::Auth;
use client::errors::SwiftError;

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

    pub fn get_container(
        &self, container: String
    ) -> GetContainer<AS> {
        GetContainer::new(self.auth.clone(), container)
    }

    pub fn get_object(
        &self, container: String, object: String
    ) -> GetObject<AS> {
        GetObject::new(self.auth.clone(), container, object)
    }

    pub fn put_object<T: Into<Body>>(
        &self, container: String, object: String, body: T
    ) -> PutObject<AS, T> {
        PutObject::new(self.auth.clone(), container, object, body)
    }
}

pub trait RunSwiftRequest {
    fn run_request(self)
        -> Result<Response, SwiftError>;

    fn add_query_param<K: Display, V: Display>(
        &self, name: &K, value: &V, query_params: &mut Vec<String>
    ) -> () {
        query_params.push(format!("{}={}", name, value));
    }

    fn add_optional_query_param<K: Display, V: Display>(
        &self, name: &K, value: &Option<V>,
        query_params: &mut Vec<String>
    ) -> () {
        match value {
            &Some(ref v) => query_params.push(
                format!("{}={}", name, v)),
            &None => ()
        }
    }
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
    fn run_request(self)
            -> Result<Response, SwiftError> {
        let mut query_params = Vec::new();
        self.add_query_param(&"limit", &self.limit, &mut query_params);
        self.add_optional_query_param(
            &"marker", &self.marker, &mut query_params);
        self.add_optional_query_param(
            &"prefix", &self.prefix, &mut query_params);
        self.add_optional_query_param(
            &"end_marker", &self.end_marker, &mut query_params);
        match self.format {
            Format::JSON => self.add_query_param(
                &"format", &"json", &mut query_params),
            Format::XML => self.add_query_param(
                &"format", &"xml", &mut query_params),
            Format::Plain => ()
        };
        let path = "?".to_string() + &query_params.join("&").to_string();

        match build_request(
            self.auth.as_ref(), Method::Get, path, self.headers.clone()
        ) {
            Ok(req) => make_request(req),
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
    fn run_request(self)
            -> Result<Response, SwiftError> {
        let path = "".to_string();
        match build_request(
                self.auth.as_ref(),
                Method::Head,
                path, self.headers.clone()) {
            Ok(req) => make_request(req),
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
    fn run_request(self)
            -> Result<Response, SwiftError> {
        let path = "".to_string();
        match build_request(
                self.auth.as_ref(),
                Method::Post,
                path, self.headers.clone()) {
            Ok(req) => make_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Get Container
 */
pub struct GetContainer<A> {
    container: String,
    marker: Option<String>,
    limit: u32,
    prefix: Option<String>,
    delimiter: Option<String>,
    end_marker: Option<String>,
    path: Option<String>,
    format: Format,
    headers: Headers,
    auth: Arc<A>
}

impl<AS: Sized+Auth> GetContainer<AS> {
    pub fn new(auth: Arc<AS>, container: String) -> GetContainer<AS> {
        GetContainer {
            container: container,
            marker: None,
            limit: 10000,
            prefix: None,
            delimiter: None,
            end_marker: None,
            path: None,
            format: Format::JSON,
            headers: Headers::new(),
            auth: auth
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for GetContainer<AS> {
    fn run_request(self)
            -> Result<Response, SwiftError> {
        let mut query_params = Vec::new();
        self.add_query_param(&"limit", &self.limit, &mut query_params);
        self.add_optional_query_param(
            &"marker", &self.marker, &mut query_params);
        self.add_optional_query_param(
            &"prefix", &self.prefix, &mut query_params);
        self.add_optional_query_param(
            &"delimiter", &self.delimiter, &mut query_params);
        self.add_optional_query_param(
            &"end_marker", &self.end_marker, &mut query_params);
        self.add_optional_query_param(
            &"path", &self.path, &mut query_params);
        match self.format {
            Format::JSON => self.add_query_param(
                &"format", &"json", &mut query_params),
            Format::XML => self.add_query_param(
                &"format", &"xml", &mut query_params),
            Format::Plain => ()
        };
        let mut path = format!("/{}?", self.container);
        path = path + &query_params.join("&").to_string();

        match build_request(
            self.auth.as_ref(), Method::Get, path, self.headers.clone()
        ) {
            Ok(req) => make_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Get Object
 */
pub struct GetObject<A> {
    container: String,
    object: String,
    multipart_manifest_get: bool,
    headers: Headers,
    auth: Arc<A>
}

impl<AS: Sized+Auth> GetObject<AS> {
    pub fn new(
        auth: Arc<AS>, container: String, object: String
    ) -> GetObject<AS> {
        GetObject {
            container: container,
            object: object,
            multipart_manifest_get: false,
            headers: Headers::new(),
            auth: auth
        }
    }
}

impl<AS: Sized+Auth> RunSwiftRequest for GetObject<AS> {
    fn run_request(self)
        -> Result<Response, SwiftError>
    {
        let mut path = format!("/{}/{}", self.container, self.object);
        if self.multipart_manifest_get {
            path = path + &format!("?{}={}", &"multipart-manifest", &"get");
        };

        match build_request(
                self.auth.as_ref(), Method::Get, path, self.headers.clone()
        ) {
            Ok(req) => make_request(req),
            Err(e) => Err(e)
        }
    }
}

/*
 * Put Object
 */
pub struct PutObject<A, T: Into<Body>> {
    container: String,
    object: String,
    multipart_manifest_put: bool,
    headers: Headers,
    body: T,
    auth: Arc<A>
}

impl<AS: Sized+Auth, T: Into<Body>> PutObject<AS, T> {
    pub fn new(
        auth: Arc<AS>, container: String, object: String, body: T
    ) -> PutObject<AS, T> {
        PutObject {
            container: container,
            object: object,
            multipart_manifest_put: false,
            headers: Headers::new(),
            body: body,
            auth: auth
        }
    }
}

impl<AS: Sized+Auth, T: Into<Body>> RunSwiftRequest for PutObject<AS, T> {
    fn run_request(self)
            -> Result<Response, SwiftError> {
        let mut path = format!("{}/{}", self.container, self.object);
        if self.multipart_manifest_put {
            path = path + &format!("?{}={}", &"multipart-manifest", &"put");
        };

        let put_req = match build_request(
            self.auth.as_ref(), Method::Put, path, self.headers.clone()
        ) {
            Ok(req) => req,
            Err(e) => return Err(e)
        };
        let put_req = put_req.body(self.body);

        make_request(put_req)
    }
}

/*
 * Helper functions
 */

fn build_request(auth: &Auth, method: Method, path: String, headers: Headers)
    -> Result<RequestBuilder, SwiftError>
{
    let req_builder = try!(
        auth.build_request(
            method, path, headers
        ).map_err(SwiftError::Auth)
    );
    Ok(req_builder)
}

fn make_request(request: RequestBuilder)
    -> Result<Response, SwiftError>
{
    let resp = match request.send() {
        Ok(r) => r,
        Err(e) => return Err(SwiftError::Http(e))
    };
    Ok(resp)
}
