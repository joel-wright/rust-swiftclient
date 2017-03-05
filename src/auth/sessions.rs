// The JSON structure requires CamelCase keys
#![allow(non_snake_case)]
use chrono::{DateTime, Duration, UTC};
use hyper::client::IntoUrl;
use hyper::header::{Headers, ContentType};
use hyper::method::Method;
use reqwest::Client;
use reqwest::RequestBuilder;
use rustc_serialize::{Encodable, json};
use std::clone::Clone;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::option::Option;
use std::result::Result;
use std::sync::Mutex;

use auth::errors::AuthError;

/*
 * Trait to be implemented by any auth object
 */
pub trait Auth {
    fn build_request(&self, m: Method, path: String, headers: Headers)
        -> Result<RequestBuilder, AuthError>;
}

/*
 * Helper methods for manipulating JSON objects
 */

fn post_json<T>(client: &Client, url: &str, payload: &T)
        -> Result<String, AuthError> where T: Encodable {
    // POSTs an encodable payload to a given URL
    let body: String = match json::encode(payload) {
        Ok(s) => s,
        Err(e) => return Err(AuthError::JsonEncode(e))
    };
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let post_res = client.post(url).body(&body[..]).headers(headers).send();
    match post_res {
        Ok(_) => {
            let mut res = post_res.unwrap();
            let mut _b = String::new();
            res.read_to_string(&mut _b).unwrap();
            Ok(_b)
        },
        Err(e) => Err(AuthError::Http(e))
    }
}

fn find_err<'j>(obj: &'j json::Json, key: &'j str) -> Result<&'j json::Json, AuthError> {
    match obj.as_object() {
        Some(_) => {
            match obj.find(key) {
                Some(r) => Ok(r),
                None => {
                    debug!("Key not found {}", key);
                    let err_msg = format!("Key not found: {}", key);
                    Err(AuthError::JsonContent(err_msg))
                }
            }
        },
        _ => {
            error!("No object could be decoded from {}", obj);
            let err_msg = format!("No object could be decoded from {}", obj);
            Err(AuthError::JsonContent(err_msg))
        }
    }
}

fn as_string<'j>(obj: &'j json::Json) -> Option<String> {
    match obj.as_string() {
        Some(s) => {
            debug!("{}", s);
            Some(String::from(s))
        },
        _ => None
    }
}

/*
 *  Keystone Auth V2
 */

#[derive(RustcEncodable)]
struct AuthRequestPasswordCredentialsV2<'s> {
    password: &'s String,
    username: &'s String
}

#[derive(RustcEncodable)]
struct AuthRequestAuthV2<'s> {
    passwordCredentials: AuthRequestPasswordCredentialsV2<'s>,
    tenantName: &'s String
}

#[derive(RustcEncodable)]
struct AuthRequestV2<'s> {
    auth: AuthRequestAuthV2<'s>,
}

#[derive(Clone)]
struct KeystoneAuthV2Token {
    token: Option<String>,
    storage_url: Option<String>,
    expires: Option<DateTime<UTC>>
}

impl KeystoneAuthV2Token {
    fn new () -> KeystoneAuthV2Token {
        KeystoneAuthV2Token{
            token: None,
            storage_url: None,
            expires: None
        }
    }
}

pub struct KeystoneAuthV2 {
    username: String,
    password: String,
    tenant: String,
    auth_url: String,
    region: Option<String>,
    client: Client,
    token: Mutex<KeystoneAuthV2Token>,
}

unsafe impl Send for KeystoneAuthV2 {}
unsafe impl Sync for KeystoneAuthV2 {}

impl KeystoneAuthV2 {
    pub fn new (username: String, password: String, tenant: String,
                auth_url: String, region: Option<String>) -> KeystoneAuthV2 {
        let client = Client::new().unwrap();
        let token = KeystoneAuthV2Token::new();
        KeystoneAuthV2 {
            username: username,
            password: password,
            tenant: tenant,
            auth_url: auth_url,
            region: region,
            client: client,
            token: Mutex::new(token)
        }
    }

    fn get_endpoint(self: &KeystoneAuthV2, endpoints: &json::Json) -> Result<Option<String>, AuthError> {
        match endpoints.as_array() {
            Some(endpoints_array) => {
                match self.region {
                    Some(ref r) => {
                        'eps: for endpoint in endpoints_array {
                            let _r = {
                                if let Ok(x) = find_err(&endpoint, "region") {x}
                                else {continue 'eps}
                            };
                            let _fr = {
                                if let Some(x) = as_string(&_r) {x}
                                else {continue 'eps}
                            };
                            if &_fr == r {
                                let _public_url = try!(find_err(&endpoint, "publicURL"));
                                let storage_url = as_string(&_public_url);
                                return Ok(storage_url)
                            }
                        }
                        error!("No region matching '{}' located", r);
                        let err_msg = format!("No region matching '{}' located", r);
                        return Err(AuthError::JsonContent(err_msg))
                    },
                    None => {
                        for endpoint in endpoints_array {
                            let _public_url = try!(find_err(&endpoint, "publicURL"));
                            let storage_url = as_string(&_public_url);
                            return Ok(storage_url)
                        }
                        error!("No endpoint for storage-url found");
                        let err_msg = format!("No endpoint for storage-url found");
                        return Err(AuthError::JsonContent(err_msg))
                    }
                }
            },
            None => {
                error!("No endpoints found");
                let err_msg = String::from("No endpoints found");
                return Err(AuthError::JsonContent(err_msg))
            }
        }
    }

    /*
     * Authenticate using supplied parameters
     */
    fn authenticate(&self, keystone_token: &mut KeystoneAuthV2Token) -> Result<(), AuthError> {
        debug!("Starting authentication");
        let auth = AuthRequestV2 {
            auth: AuthRequestAuthV2 {
                passwordCredentials: AuthRequestPasswordCredentialsV2 {
                    password: &self.password,
                    username: &self.username
                },
            tenantName: &self.tenant
        }};

        let _au = &format!("{}/{}", &self.auth_url, "tokens")[..];
        let response = try!(post_json(&self.client, _au, &auth));
        let response_object: json::Json = match json::Json::from_str(&response) {
            Ok(j) => j,
            Err(e) => return Err(AuthError::JsonDecode(e))
        };

        // Get the expiry time and access token
        let access: &json::Json = try!(find_err(&response_object, "access"));
        let token: &json::Json = try!(find_err(access, "token"));
        let token_id: &json::Json = try!(find_err(token, "id"));
        let expires: &json::Json = try!(find_err(token, "expires"));

        // Get the service catalogue and find the object store
        let catalogue: &json::Json = try!(find_err(access, "serviceCatalog"));
        let mut storage_url: Option<String> = None;

        match catalogue.as_array() {
            Some(catalogue_array) => {
                'catalogue: for service in catalogue_array {
                    let _type = try!(find_err(service, "type"));
                    match _type.as_string() {
                        Some("object-store") => {
                            let endpoints = try!(find_err(&service, "endpoints"));
                            storage_url = try!(self.get_endpoint(&endpoints));
                        },
                        _ => ()
                    };
                }
            },
            None => ()
        };
        match storage_url {
            Some(_) => {
                keystone_token.storage_url = storage_url;
                keystone_token.token = as_string(&token_id);
                match expires.as_string() {
                    Some(s) => {
                        match s.parse::<DateTime<UTC>>() {
                            Ok(d) => keystone_token.expires = Some(d),
                            _ => {
                                error!("Failed to parse auth token expiry time");
                                let err_msg = String::from("Failed to parse auth token expiry time");
                                return Err(AuthError::JsonContent(err_msg))
                            }
                        };
                    },
                    _ => {
                        error!("Failed to parse auth token expiry time");
                        let err_msg = String::from("Failed to parse auth token expiry time");
                        return Err(AuthError::JsonContent(err_msg))
                    }
                };
                Ok(())
            },
            _ => {
                error!("Failed to find object-store in catalogue");
                let err_msg = String::from("Failed to find object-store in catalogue");
                Err(AuthError::JsonContent(err_msg))
            }
        }
    }

    unsafe fn get_token(&self) -> Result<(), AuthError> {
        {
            match self.token.try_lock() {
                Ok(mut keystone_token) => {
                    match keystone_token.token {
                        Some(_) => {
                            match keystone_token.expires {
                                Some(datetime) => {
                                    let now = UTC::now();
                                    let d: DateTime<UTC> = datetime - Duration::hours(1);
                                    if now.lt(&d) {    // If the token expires more than
                                        return Ok(())  // an hour from now, just return it
                                    }
                                },
                                None => ()  // No expiry time means no token
                            };
                        },
                        None => ()  // No token means we need to auth
                    };
                    // If we get here then we have a lock but no valid token, so auth
                    return self.authenticate(keystone_token.deref_mut())
                },
                Err(_) => return Ok(())  // If we can't get the lock, just assume that
                                         // another thread is authenticating - build_request
                                         // will then lock until it's complete
            }
        };
    }
}

/*
 * Get auth token, authenticating if necessary
 */
header! { (XAuthToken, "X-Auth-Token") => [String] }

impl Auth for KeystoneAuthV2 {
    fn build_request(&self, m: Method, path: String, mut headers: Headers)
            -> Result<RequestBuilder, AuthError> {
        // Make sure we have a valid auth token
        unsafe {
            match self.get_token() {
                Ok(()) => (),
                Err(e) => return Err(e)
            }
        };
        // Either the current thread got the token and it's ready, or
        // another thread is getting the token and we have to wait
        let keystone_token: KeystoneAuthV2Token = match self.token.lock() {
            Ok(t) => t.deref().clone(),
            Err(_) => {
                error!("Failed to grab the current access token");
                let err_msg = String::from("Locking token failed");
                return Err(AuthError::Fail(err_msg))
            }
        };
        let token = match keystone_token.token {
            Some(t) => t,
            None => {
                error!("No current access token found");
                let err_msg = String::from("No current access token found");
                return Err(AuthError::Fail(err_msg))
            }
        };
        let storage_base_url: &String = match keystone_token.storage_url {
            Some(ref u) => u,
            None => {
                error!("No storage base URL found");
                let err_msg = String::from("No storage base URL found");
                return Err(AuthError::Fail(err_msg))
            }
        };
        let mut url = String::from("");
        url.push_str(storage_base_url);
        url.push_str(&path);
        debug!("Request base URL: {}", url);
        match url.into_url() {
            Ok(_u) => {
                headers.set(XAuthToken(token));
                return Ok(self.client.request(m, _u).headers(headers))
            }
            _ => {
                error!("Failed to parse request base URL: {}", url);
                let err_msg = String::from("Failed to parse base request URL");
                return Err(AuthError::Fail(err_msg))
            }
        }
    }
}
