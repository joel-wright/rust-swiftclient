extern crate rustc_serialize;
extern crate docopt;
extern crate chrono;
extern crate hyper;
extern crate rust_swiftclient;

use docopt::Docopt;
use hyper::status::StatusCode;
use rust_swiftclient::auth::sessions::KeystoneAuthV2;
use rust_swiftclient::auth::request::AuthRequest;
use std::env;
use std::thread;
use std::sync::Arc;

const USAGE: &'static str = "
Usage: swift [options] [<command>]

Options:
    -U, --user=<user>          username for auth
    -T, --tenant=<tenant>      tenant name for auth
    -A, --auth-url=<url>       URL of the auth system
    -P, --password=<password>  password for auth
    -R, --region=<region>      region for auth
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

fn get_arg(arg: Option<String>, os_var: String) -> Option<String> {
    match env::var(os_var) {
        Ok(v) => match arg {
            Some(u) => Some(u),
            None => Some(v)
        },
        Err(_) => match arg {
            Some(u) => Some(u),
            None => None
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|dopt| dopt.decode())
                            .unwrap_or_else(|e| e.exit());

    let user = get_arg(args.flag_user, String::from("OS_USERNAME")).unwrap();
    let pwd = get_arg(args.flag_password, String::from("OS_PASSWORD")).unwrap();
    let tenant = get_arg(args.flag_tenant, String::from("OS_TENANT_NAME")).unwrap();
    let url = get_arg(args.flag_auth_url, String::from("OS_AUTH_URL")).unwrap();
    let region = args.flag_region;

    let ksauth = KeystoneAuthV2::new(user, pwd, tenant, url, region);
    let auth_request = Arc::new(AuthRequest::new(ksauth));

    let thread_action = {
        let ar = auth_request.clone();
        let thread_action = thread::spawn(move || {
            let path = String::from("/jjw");
            match ar.head(&path) {
                Ok(resp) => {
                    assert_eq!(resp.status, StatusCode::NoContent);
                    for item in resp.headers.iter() {
                        println!("{:?}", item);
                    }
                }
                Err(s) => println!("{}", s)
            };
        });
        thread_action
    };

    {
        let path = String::from("/jjw/loadsafiles/006224");
        let ar = auth_request.clone();
        match ar.head(&path) {
            Ok(resp) => {
                for header in resp.headers.iter() {
                    println!(
                        "{0:?}: {1:?}",
                        header.name(),
                        header.value_string()
                    );
                }
            }
            Err(s) => println!("{}", s)
        };
    }

    let result = thread_action.join();
    match result {
        Err(_) => println!("All went boom"),
        _ => ()
    }
}
