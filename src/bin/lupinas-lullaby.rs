extern crate clap;

use reqwest::{blocking::Client, StatusCode};
use std::{error::Error, fs::OpenOptions};

pub fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(
            clap::Arg::with_name("file")
                .long("file")
                .help("The Swagger file to parse.")
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("url")
                .long("url")
                .help("The URL to the Swagger file. Must be a URL to a JSON/YAML resource")
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("auth-user")
                .long("auth-user")
                .help(r##"The basic authentication password payload to pass along."##)
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("auth-password")
                .long("auth-password")
                .help(r##"The basic authentication username payload to pass along."##)
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("write")
                .long("write")
                .help(
                    r##"The destination file to write to.
If this value is not specified, it will simply write to stdout.
"##,
                )
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("stdin")
                .long("stdin")
                .help("Accepts input from stdin")
                .required(false)
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("skip-invalid-types")
                .long("skip-invalid-types")
                .help("Skip invalid types like overwriting standard JS and empty types")
                .required(false)
                .takes_value(false),
        )
        .get_matches();
    let spec = if let Some(file) = matches.value_of("file") {
        openapi::from_path(file)
    } else if let Some(url) = matches.value_of("url") {
        let auth_username = matches.value_of("auth-user").unwrap_or("");
        let auth_password = matches.value_of("auth-password");
        let res = Client::new()
            .get(url)
            .basic_auth(auth_username, auth_password)
            .send()
            .unwrap();
        if res.status() == StatusCode::OK {
            let res = res.bytes().unwrap();
            openapi::from_bytes(&res)
        } else {
            eprintln!("Http request failed with response:\n{:#?}", res);
            return Ok(());
        }
    } else if let Some(_) = matches.value_of("stdin") {
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer).unwrap();
        let result = serde_yaml::from_str(&buffer).unwrap();
        result
    } else {
        eprintln!("Please enter an input with '--input' or '--stdin'. See help for more info.");
        return Ok(());
    };
    let skip = matches.is_present("skip-invalid-types");
    let stringified = openapi::use_spec(&spec, skip);
    if let Some(write) = matches.value_of("write") {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(write)
            .unwrap();
        std::io::Write::write_all(&mut file, stringified.as_bytes()).unwrap();
        std::io::Write::flush(&mut file).unwrap();
    } else {
        println!("{}", stringified);
    }
    Ok(())
}
