// The JSON structure requires CamelCase keys
#![allow(non_snake_case)]
extern crate hyper;
extern crate chrono;
extern crate rustc_serialize;
extern crate url;

use self::hyper::Client;
use self::hyper::header::{Headers, ContentType};
use self::hyper::method::Method;
use self::hyper::client::{IntoUrl, RequestBuilder};
use chrono::{DateTime, Duration, UTC};
use rustc_serialize::{Encodable, json};
use std::io::Read;
use std::option::Option;
use std::result::Result;
use std::sync::Mutex;
use std::error;
use std::fmt;

/////////////////////////////////////
// Enum for containing auth methods
/////////////////////////////////////

//pub enum Auth {
//    KeystoneAuthV2
//}

////////////////////
// Errors for Auth
////////////////////
#[derive(Debug)]
pub enum AuthError {
    Http(hyper::error::Error),
    JsonEncode(rustc_serialize::json::EncoderError),
    JsonDecode(rustc_serialize::json::ParserError),
    JsonContent(String),
    Fail(String)
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthError::Http(ref err) => write!(f, "HTTP error: {}", err),
            AuthError::JsonEncode(ref err) => write!(f, "JSON Encode error: {}", err),
            AuthError::JsonDecode(ref err) => write!(f, "JSON Decode error: {}", err),
            AuthError::JsonContent(ref s) => write!(f, "JSON Content error: {}", s),
            AuthError::Fail(ref s) => write!(f, "Fail: {}", s),
        }
    }
}

impl error::Error for AuthError {
    fn description(&self) -> &str {
        match *self {
            AuthError::Http(ref err) => err.description(),
            AuthError::JsonEncode(ref err) => err.description(),
            AuthError::JsonDecode(ref err) => err.description(),
            AuthError::JsonContent(ref s) => s,
            AuthError::Fail(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AuthError::Http(ref err) => Some(err),
            AuthError::JsonEncode(ref err) => Some(err),
            AuthError::JsonDecode(ref err) => Some(err),
            AuthError::JsonContent(_) => None,
            AuthError::Fail(_) => None,
        }
    }
}

///////////////////////////////////////////////
// Trait to be implemented by any auth object
///////////////////////////////////////////////

pub trait Auth {
    fn build_request<'a>(&'a self, m: Method, path:&'a String)
        -> Result<RequestBuilder<'a>, AuthError>;
}

/////////////////////////////////////////////////
// Helper methods for manipulating JSON objects
/////////////////////////////////////////////////

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
                    //println!("Key not found {}", key);
                    let err_msg = format!("Key not found: {}", key);
                    Err(AuthError::JsonContent(err_msg))
                }
            }
        },
        _ => {
            let err_msg = format!("No object could be decoded");
            Err(AuthError::JsonContent(err_msg))
        }
    }
}

fn as_string<'j>(obj: &'j json::Json) -> Option<String> {
    match obj.as_string() {
        Some(s) => {
            println!("{}", s);
            Some(String::from(s))
        },
        _ => None
    }
}

//////////////////////
//  Keystone Auth V2
//////////////////////

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

impl KeystoneAuthV2 {
    pub fn new (username: String, password: String, tenant: String,
                auth_url: String, region: Option<String>) -> KeystoneAuthV2 {
        let client = Client::new();
        let token = KeystoneAuthV2Token::new();
        KeystoneAuthV2 {
            username: username,
            password: password,
            tenant: tenant,
            auth_url: auth_url,
            region: region,
            token: Mutex::new(token),
            client: client
        }
    }

    fn get_endpoint(self: &KeystoneAuthV2, endpoints: &json::Json) -> Result<Option<String>, AuthError> {
        match endpoints.as_array() {
            Some(endpoints_array) => {
                match self.region {
                    Some(ref r) => {
                        'eps: for endpoint in endpoints_array {
                            let _r = if let Ok(x) = find_err(&endpoint, "region") {x}
                                     else {continue 'eps};
                            let _fr = if let Some(x) = as_string(&_r) {x}
                                     else {continue 'eps};
                            if &_fr == r {
                                let _public_url = try!(find_err(&endpoint, "publicURL"));
                                let storage_url = as_string(&_public_url);
                                return Ok(storage_url)
                            }
                        }
                        let err_msg = format!("No region matching '{}' located", r);
                        return Err(AuthError::JsonContent(err_msg))
                    },
                    None => {
                        for endpoint in endpoints_array {
                            let _public_url = try!(find_err(&endpoint, "publicURL"));
                            let storage_url = as_string(&_public_url);
                            return Ok(storage_url)
                        }
                        let err_msg = format!("No Endpoint for storage-url found");
                        return Err(AuthError::JsonContent(err_msg))
                    }
                }
            },
            None => {
                let err_msg = String::from("No Endpoints Found");
                return Err(AuthError::JsonContent(err_msg))
            }
        }
    }

    ///////////////////////////////////////////
    // Authenticate using supplied parameters
    ///////////////////////////////////////////
    fn authenticate(&self) -> Result<(), AuthError> {
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
                let mut keystone_token = self.token.lock().unwrap();
                keystone_token.storage_url = storage_url;
                keystone_token.token = as_string(&token_id);
                match expires.as_string() {
                    Some(s) => {
                        match s.parse::<DateTime<UTC>>() {
                            Ok(d) => keystone_token.expires = Some(d),
                            _ => {
                                let err_msg = String::from("Failed to parse token expiry time");
                                return Err(AuthError::JsonContent(err_msg))
                            }
                        };
                    },
                    _ => {
                        let err_msg = String::from("Failed to parse token expiry time");
                        return Err(AuthError::JsonContent(err_msg))
                    }
                };
                Ok(())
            },
            _ => {
                let err_msg = String::from("Failed to find object-store in catalogue");
                Err(AuthError::JsonContent(err_msg))
            }
        }
    }

    unsafe fn get_token(&self) -> Result<(), AuthError> {
        {
            match self.token.try_lock() {
                Ok(keystone_token) => {
                    match keystone_token.expires {
                        Some(datetime) => {
                            let now = UTC::now();
                            let d: DateTime<UTC> = datetime - Duration::hours(1);
                            if now.lt(&d) {  // If the token expires more than an hour from now
                                             // just return it.
                                match keystone_token.token {
                                    Some(_) => return Ok(()),
                                    None => ()  // No token means we need to auth
                                }
                            }
                        },
                        None => ()  // No expiry time means no token
                    }
                },
                Err(_) => return Ok(())  // If we can't get the lock, just assume that
                                         // another thread is authenticating
            }
        };
        // If we get here we need to try to authenticate again
        return self.authenticate();
    }
}

////////////////////////////////////////////////
// Get auth token, authenticating if necessary
////////////////////////////////////////////////
header! { (XAuthToken, "X-Auth-Token") => [String] }

impl Auth for KeystoneAuthV2 {
    fn build_request<'a>(&'a self, m: Method, path:&'a String)
            -> Result<RequestBuilder<'a>, AuthError> {
        // Make sure we have a valid auth token
        unsafe {
            match self.get_token() {
                Ok(()) => (),
                Err(e) => return Err(e)
            }
        };
        let keystone_token = match self.token.lock() {
            Ok(t) => t,
            Err(_) => {
                let err_msg = String::from("Locking token failed");
                return Err(AuthError::Fail(err_msg))
            }
        };
        let token = match keystone_token.token.clone() {
            Some(t) => t,
            None => {
                let err_msg = String::from("Cloning token failed");
                return Err(AuthError::Fail(err_msg))
            }
        };
        let _us: &String = match keystone_token.storage_url {
            Some(ref u) => u,
            None => {
                let err_msg = String::from("No base URL found");
                return Err(AuthError::Fail(err_msg))
            }
        };
        let mut url = String::from("");
        url.push_str(_us);
        url.push_str(path);
        println!("{}", url);
        match url.into_url() {
            Ok(_u) => {
                let mut headers = Headers::new();
                headers.set(XAuthToken(token));
                return Ok(self.client.request(m, _u).headers(headers))
            }
            _ => {
                let err_msg = String::from("Failed to parse URL");
                return Err(AuthError::Fail(err_msg))
            }
        }
    }
}
