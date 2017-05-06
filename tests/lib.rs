#[macro_use]
extern crate lazy_static;
extern crate hyper;
extern crate rust_swiftclient;

use std::env;
use std::io;
use std::sync::Arc;

use rust_swiftclient::auth::sessions::KeystoneAuthV2;
use rust_swiftclient::client::request::{
    RunSwiftRequest, SwiftConnection
};

fn get_arg(os_var: String) -> String {
    match env::var(&os_var) {
        Ok(v) => v,
        Err(_) => panic!("Cannot find env var: {}", &os_var)
    }
}

lazy_static! {
    static ref CONTAINER: String = get_arg(String::from("TEST_CONTAINER"));
    static ref OBJECT: String = get_arg(String::from("TEST_OBJECT"));
    static ref SWIFT: Arc<SwiftConnection<KeystoneAuthV2>> =
        Arc::new(
            SwiftConnection::new(
                KeystoneAuthV2::new(
                    get_arg(String::from("OS_USERNAME")),
                    get_arg(String::from("OS_PASSWORD")),
                    get_arg(String::from("OS_PROJECT_NAME")),
                    get_arg(String::from("OS_AUTH_URL")),
                    None
                )
            )
        );
}

#[test]
fn head_account() {
    let sw = SWIFT.clone();
    let ga = sw.get_account();
    match ga.run_request() {
        Ok(resp) => {
            let cc = String::from("X-Account-Container-Count");
            let oc = String::from("X-Account-Object-Count");
            let bu = String::from("X-Account-Bytes-Used");
            for header in vec![cc, oc, bu] {
                match resp.headers().get_raw(&header) {
                    Some(_) => {},
                    None => panic!("Expected header not found: {}", &header)
                }
            }
        },
        Err(s) => panic!("{}", s)
    };
}

#[test]
fn get_account() {
    let sw = SWIFT.clone();
    let ga = sw.get_account();
    match ga.run_request() {
        Ok(mut resp) => {
            let mut body_vec: Vec<u8> = vec![];
            match io::copy(&mut resp, &mut body_vec) {
                // TODO: test contents
                Ok(_) => {},
                Err(e) => panic!("{}", e)
            };
        },
        Err(s) => panic!("{}", s)
    };
}

#[test]
fn get_container() {
    let sw = SWIFT.clone();
    // TODO: define some proper test environment
    let container = CONTAINER.clone();
    let gc = sw.get_container(container);
    match gc.run_request() {
        Ok(mut resp) => {
            let mut body_vec: Vec<u8> = vec![];
            match io::copy(&mut resp, &mut body_vec) {
                // TODO: test contents
                Ok(_) => {},
                Err(e) => panic!("{}", e)
            };
        },
        Err(s) => panic!("{}", s)
    };
}

#[test]
fn get_object() {
    let sw = SWIFT.clone();
    // TODO: define some proper test environment
    let container = CONTAINER.clone();
    let object = OBJECT.clone();
    let go = sw.get_object(container, object);
    match go.run_request() {
        Ok(mut resp) => {
            let mut body_vec: Vec<u8> = vec![];
            match io::copy(&mut resp, &mut body_vec) {
                // TODO: test contents
                Ok(_) => {},
                Err(e) => panic!("{}", e)
            };
        },
        Err(s) => panic!("{}", s)
    };
}