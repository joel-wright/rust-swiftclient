// The JSON structure requires CamelCase keys
#![allow(non_snake_case)]
extern crate hyper;
extern crate chrono;
extern crate rustc_serialize;

use self::hyper::Client;
use self::hyper::header::{Headers, ContentType};
use chrono::{DateTime, Duration, UTC};
use rustc_serialize::{Encodable, json};
use std::io::Read;
use std::option::Option;
use std::result::Result;

///////////////////////////////////////////////
// Trait to be implemented by any auth object
///////////////////////////////////////////////

pub trait Authenticate {
    fn get_token(&mut self) -> Result<&String, String>;
}

/////////////////////////////////////////////////
// Helper methods for manipulating JSON objects
/////////////////////////////////////////////////

fn post_json<T>(client: &Client, url: &str, payload: &T) -> Result<String, String> where T: Encodable {
    // POSTs an encodable payload to a given URL
    let body: String = match json::encode(payload) {
        Ok(s) => s,
        Err(_) => return Err(String::from("Failed to encode JSON body"))
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
        Err(_) => Err(String::from("POST request failed"))
    }
}

fn find_err<'j>(obj: &'j json::Json, key: &'j str) -> Result<&'j json::Json, String> {
    match obj.as_object() {
        Some(_) => {
            match obj.find(key) {
                Some(r) => Ok(r),
                None => {
                    //println!("Key not found {}", key);
                    let err_msg = format!("Key not found: {}", key);
                    Err(err_msg)
                }
            }
        },
        _ => Err(String::from("Not a JSON object"))
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

pub struct KeystoneAuthV2 {
    username: String,
    password: String,
    tenant: String,
    auth_url: String,
    region: Option<String>,
    token: Option<String>,
    storage_url: Option<String>,
    expires: Option<DateTime<UTC>>,
    client: Client
}

impl KeystoneAuthV2 {
    pub fn new (username: String, password: String, tenant: String,
                auth_url: String, region: Option<String>) -> KeystoneAuthV2 {
        let client = Client::new();
        KeystoneAuthV2 {
            username: username,
            password: password,
            tenant: tenant,
            auth_url: auth_url,
            region: region,
            token: None,
            storage_url: None,
            expires: None,
            client: client
        }
    }

    fn get_endpoint(self: &KeystoneAuthV2, endpoints: &json::Json) -> Result<Option<String>, String> {
        match endpoints.as_array() {
            Some(endpoints_array) => {
                match self.region {
                    Some(ref r) => {
                        'eps: for endpoint in endpoints_array {
                            let _r = if let Ok(x) = find_err(&endpoint, "region") {x} else {continue 'eps};
                            let _fr = if let Some(x) = as_string(&_r) {x} else {continue 'eps};
                            if &_fr == r {
                                let _public_url = try!(find_err(&endpoint, "publicURL"));
                                let storage_url = as_string(&_public_url);
                                return Ok(storage_url)
                            }
                        }
                        return Err(format!("No region matching '{}' located", r))
                    },
                    None => {
                        for endpoint in endpoints_array {
                            let _public_url = try!(find_err(&endpoint, "publicURL"));
                            let storage_url = as_string(&_public_url);
                            return Ok(storage_url)
                        }
                        return Err(format!("No Endpoint for storage-url found"))
                    }
                }
            },
            None => return Err(String::from("No Endpoints Found"))
        }
    }

    ///////////////////////////////////////////
    // Authenticate using supplied parameters
    ///////////////////////////////////////////
    fn authenticate(self: &mut KeystoneAuthV2) -> Result<(), String> {
        let auth = AuthRequestV2 {
            auth: AuthRequestAuthV2 {
                passwordCredentials: AuthRequestPasswordCredentialsV2 {
                    password: &self.password,
                    username: &self.username
                },
            tenantName: &self.tenant
        }};

        let response = try!(post_json(&self.client, &self.auth_url[..], &auth));
        let response_object: json::Json = match json::Json::from_str(&response) {
            Ok(j) => j,
            Err(_) => return Err(String::from("Failed to decode JSON object from str"))
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
                self.storage_url = storage_url;
                self.token = as_string(&token_id);
                match expires.as_string() {
                    Some(s) => {
                        match s.parse::<DateTime<UTC>>() {
                            Ok(d) => self.expires = Some(d),
                            Err(_) => self.expires = None
                        };
                    },
                    _ => self.expires = None
                };
                Ok(())
            },
            _ => Err(String::from("Failed to find object-store in catalogue"))
        }
    }
}

////////////////////////////////////////////////
// Get auth token, authenticating if necessary
////////////////////////////////////////////////
impl Authenticate for KeystoneAuthV2 {
    fn get_token(&mut self) -> Result<&String, String> {
        match self.expires {
            Some(datetime) => {
                let now = UTC::now();
                let d: DateTime<UTC> = datetime - Duration::hours(1);
                if now.gt(&d) {
                    try!(self.authenticate());
                }
            },
            None => {
                try!(self.authenticate());
            }
        }
        match self.token {
            Some(ref t) => Ok(t),
            None => Err(String::from("No token found"))
        }
    }
}
