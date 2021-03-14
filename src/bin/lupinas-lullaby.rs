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
            clap::Arg::with_name("skip-empty-types")
                .long("skip-empty-types")
                .help(r#"Skip empty types because some linter will complain.
Possibly only relevant in languages with structural typing e.g. TypeScript."#)
                .required(false)
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("skip-type-name")
                .long("skip-type-name")
                .help(r#"Skip types with the given name.
Useful if the swagger file overwrites some implicitly imported classes or its messing up type checking.
Takes multiple occurences."#)
                .required(false)
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
        )
        .get_matches();
    let spec = if let Some(file) = matches.value_of("file") {
        openapi::from_path(file)
    } else if let Some(url) = matches.value_of("url") {
        let auth_username = matches.value_of("auth-user");
        let auth_password = matches.value_of("auth-password");
        let mut res = Client::new().get(url);
        if let Some(auth_username) = auth_username {
            res = res.basic_auth(auth_username, auth_password);
        }
        let res = res.send().unwrap();
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
    let skip = matches.is_present("skip-empty-types");
    let typenames_to_skip = matches
        .values_of("skip-type-name")
        .unwrap_or_default()
        .collect::<Vec<_>>();
    let stringified = openapi::use_spec(&spec, skip, typenames_to_skip);
    if let Some(write) = matches.value_of("write") {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(write)
            .unwrap();
        std::io::Write::write_all(&mut file, stringified.as_bytes()).unwrap();
        std::io::Write::flush(&mut file).unwrap();
    } else {
        println!("{}", stringified);
    }
    Ok(())
}
