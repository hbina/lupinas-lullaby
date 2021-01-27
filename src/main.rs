extern crate clap;

mod openapi;

use reqwest::{blocking::Client, StatusCode};
use std::{error::Error, fs::OpenOptions};

pub fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
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
            clap::Arg::with_name("auth_user")
                .long("auth_user")
                .help(r##"The basic authentication password payload to pass along."##)
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("auth_password")
                .long("auth_password")
                .help(r##"The basic authentication username payload to pass along."##)
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("outfile")
                .long("outfile")
                .help(
                    r##"The destination file to write to.
If this value is not specified, it will simply write to stdout.
"##,
                )
                .required(false)
                .takes_value(true),
        )
        .get_matches();
    let result = if let Some(file) = matches.value_of("file") {
        println!("Opening file `{}`", file);
        openapi::use_spec(&openapi::from_path(file))
    } else if let Some(url) = matches.value_of("url") {
        let auth_username = matches.value_of("auth_user").unwrap_or("");
        let auth_password = matches.value_of("auth_password");
        let res = Client::new()
            .get(url)
            .basic_auth(auth_username, auth_password)
            .send()?;
        if res.status() == StatusCode::OK {
            let res = res.bytes()?;
            openapi::use_spec(&openapi::from_bytes(&res))
        } else {
            panic!("HTTP GET response returned error.\n{:#?}", res)
        }
    } else {
        panic!("Please provide at least the file or the URL to parse!")
    };
    if let Some(outfile) = matches.value_of("outfile") {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(outfile)?;
        std::io::Write::write_all(&mut file, result.as_bytes())?;
    } else {
        println!("{}", result);
    }
    Ok(())
}