// Allow zero pointers for lazy_static. Otherwise clippy will complain.
#![allow(unknown_lints)]

#[macro_use]
extern crate clap;
extern crate liquid;

#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "serde_yaml")]
extern crate serde_yaml;

use std::ffi;
use std::fs;
use std::io::Write;
use std::path;

#[derive(Copy, Clone, Debug, derive_more::Display, derive_more::From, derive_more::Constructor)]
#[display(fmt = "{}", msg)]
struct Error {
    msg: &'static str,
}

impl std::error::Error for Error {}

fn option<'a>(name: &'a str, value: &'a str) -> clap::Arg<'a, 'a> {
    clap::Arg::with_name(name).long(name).value_name(value)
}

#[cfg(feature = "serde_yaml")]
fn load_yaml(path: &path::Path) -> Result<liquid::value::Value, Box<dyn std::error::Error>> {
    let f = fs::File::open(path)?;
    serde_yaml::from_reader(f).map_err(|e| e.into())
}

#[cfg(not(feature = "serde_yaml"))]
fn load_yaml(_path: &path::Path) -> Result<liquid::value::Value, Box<dyn std::error::Error>> {
    Err(Error::new("yaml is unsupported"))?
}

#[cfg(feature = "serde_json")]
fn load_json(path: &path::Path) -> Result<liquid::value::Value, Box<dyn std::error::Error>> {
    let f = fs::File::open(path)?;
    serde_json::from_reader(f).map_err(|e| e.into())
}

#[cfg(not(feature = "serde_json"))]
fn load_json(_path: &path::Path) -> Result<liquid::value::Value, Box<dyn std::error::Error>> {
    Err(Error::new("json is unsupported"))?
}

fn build_context(path: &path::Path) -> Result<liquid::value::Object, Box<dyn std::error::Error>> {
    let extension = path.extension().unwrap_or_else(|| ffi::OsStr::new(""));
    let value = if extension == ffi::OsStr::new("yaml") {
        load_yaml(path)
    } else if extension == ffi::OsStr::new("yaml") {
        load_json(path)
    } else {
        Err(Error::new("Unsupported file type"))?
    }?;
    let value = match value {
        liquid::value::Value::Object(o) => Ok(o),
        _ => Err(Error::new("File must be an object")),
    }?;

    Ok(value)
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let matches = clap::App::new("liquidate")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(option("input", "LIQUID").required(true))
        .arg(option("output", "TXT"))
        .arg(option("context", "YAML"))
        .get_matches_safe()?;

    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .jekyll_filters()
        .build()
        .expect("should succeed without partials");
    let template_path = matches
        .value_of("input")
        .map(path::PathBuf::from)
        .expect("Parameter was required");
    let template = parser.parse_file(template_path)?;

    let data = matches
        .value_of("context")
        .map(|s| {
            let p = path::PathBuf::from(s);
            build_context(p.as_path())
        })
        .map_or(Ok(None), |r| r.map(Some))?
        .unwrap_or_else(liquid::value::Object::new);
    let output = template.render(&data)?;
    match matches.value_of("output") {
        Some(path) => {
            let mut out = fs::File::create(path::PathBuf::from(path))?;
            out.write_all(output.as_bytes())?;
        }
        None => {
            println!("{}", output);
        }
    }

    Ok(0)
}

fn main() {
    let code = run().unwrap();
    std::process::exit(code);
}
