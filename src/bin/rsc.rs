extern crate rustc_serialize;
extern crate docopt;
extern crate chrono;
extern crate hyper;
extern crate rust_swiftclient;

use docopt::Docopt;

use std::env;
use std::io;
use std::process::exit;
use std::thread;
use std::sync::Arc;

use rust_swiftclient::auth::sessions::KeystoneAuthV2;
use rust_swiftclient::client::request::{
    RunSwiftRequest, SwiftConnection
};

const USAGE: &'static str = "
Usage: swift [options] [<command>]

Options:
    -U, --user=<user>          username (must be specified or set in env[$OS_USERNAME])
    -T, --project=<project>    project name (must be specified or set in env[$OS_PROJECT])
    -A, --auth-url=<url>       URL of the auth system (must be specified or set in env[$OS_AUTH_URL])
    -P, --password=<password>  password (must be specified or set in env[$OS_PASSWORD])
    -R, --region=<region>      region (optional, can be set in env[$OS_REGION_NAME])
    -h, --help                 display this help and exit
    -v, --version              output version information and exit
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_command: Option<String>,
    flag_user: Option<String>,
    flag_tenant: Option<String>,
    flag_auth_url: Option<String>,
    flag_password: Option<String>,
    flag_region: Option<String>
}

// I think this stuff needs to be moved to a separate options handler
fn get_arg(arg: Option<String>, os_var: String) -> String {
    match arg {
        Some(u) => u,
        None => match env::var(os_var) {
            Ok(v) => v,
            Err(_) => {
                println!("{}", USAGE);
                exit(1);
            }
        }
    }
}

fn get_optional_arg(arg: Option<String>, os_var: String) -> Option<String> {
    match arg {
        Some(u) => Some(u),
        None => match env::var(os_var) {
            Ok(v) => Some(v),
            Err(_) => None
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|dopt| dopt.decode())
                            .unwrap_or_else(|e| e.exit());

    let user = get_arg(args.flag_user, String::from("OS_USERNAME"));
    let pwd = get_arg(args.flag_password, String::from("OS_PASSWORD"));
    let tenant = get_arg(args.flag_tenant, String::from("OS_PROJECT_NAME"));
    let url = get_arg(args.flag_auth_url, String::from("OS_AUTH_URL"));
    let region = get_optional_arg(args.flag_region, String::from("OS_REGION_NAME"));

    let ksauth = KeystoneAuthV2::new(user, pwd, tenant, url, region);
    let swift: Arc<SwiftConnection<KeystoneAuthV2>> =
        Arc::new(SwiftConnection::new(ksauth));

    let get_account_action = {
        let sw = swift.clone();
        let thread_action = thread::spawn(move || {
            let ga = sw.get_account();
            match ga.run_request() {
                Ok(resp) => {
                    for header in resp.headers().iter() {
                        println!(
                            "{0:?}: {1:?}",
                            header.name(),
                            header.value_string()
                        );
                    }
                },
                Err(s) => {
                    println!("{}", s);
                }
            };
        });
        thread_action
    };

    let get_object_action = {
        let sw = swift.clone();
        let thread_action = thread::spawn(move || {
            let go = sw.get_object(
                String::from("jjw"),
                String::from("hello_world")
            );
            match go.run_request() {
                Ok(mut resp) => {
                    for header in resp.headers().iter() {
                        println!(
                            "object - {0:?}: {1:?}",
                            header.name(),
                            header.value_string()
                        );
                    };
                    let mut body_vec: Vec<u8> = vec![];
                    match io::copy(&mut resp, &mut body_vec) {
                        Ok(bytes) => {
                            let body = String::from_utf8(body_vec).unwrap();
                            println!("{} (bytes: {})", body, bytes);
                        },
                        Err(e) => {
                            println!("{}", e);
                            ()
                        }
                    };
                },
                Err(s) => {
                    println!("{}", s);
                }
            };
        });
        thread_action
    };

    get_account_action.join();
    get_object_action.join();
}
