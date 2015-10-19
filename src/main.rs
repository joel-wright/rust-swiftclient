extern crate rustc_serialize;
extern crate docopt;
extern crate chrono;

use docopt::Docopt;

mod auth;

use auth::{KeystoneAuthV2, Authenticate};

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
    flag_user: String,
    flag_tenant: String,
    flag_auth_url: String,
    flag_password: String,
    flag_region: Option<String>
}

fn main() {
    println!("STARTING AUTH");
    let args: Args = Docopt::new(USAGE)
                            .and_then(|dopt| dopt.decode())
                            .unwrap_or_else(|e| e.exit());

    let user = args.flag_user;
    let pwd = args.flag_password;
    let tenant = args.flag_tenant;
    let url = args.flag_auth_url;
    let region = args.flag_region;

    let mut ksauth = KeystoneAuthV2::new(user, pwd, tenant, url, region);
    let token = ksauth.get_token();
    match token {
        Ok(t) => println!("Success: {}", t),
        Err(e) => println!("Fail: {}", e)
    }

    println!("AUTH FINISHED")
}
