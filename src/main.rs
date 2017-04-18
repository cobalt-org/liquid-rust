#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate liquid;
extern crate toml;

use std::collections;
use std::fs;
use std::io;
use std::io::{Write, Read};
use std::path;
use liquid::Renderable;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

error_chain! {

    links {
    }

    foreign_links {
        Clap(clap::Error);
        Io(io::Error);
        Liquid(liquid::Error);
        Toml(toml::de::Error);
    }

    errors {
    }
}

fn option<'a>(name: &'a str, value: &'a str) -> clap::Arg<'a, 'a> {
    clap::Arg::with_name(name).long(name).value_name(value)
}

fn convert_value(toml_value: &toml::Value) -> Result<liquid::Value> {
    match *toml_value {
        toml::Value::String(ref s) => Ok(liquid::Value::Str(s.to_string())),
        toml::Value::Integer(n) => Ok(liquid::Value::Num(n as f32)),
        toml::Value::Float(n) => Ok(liquid::Value::Num(n as f32)),
        toml::Value::Boolean(b) => Ok(liquid::Value::Bool(b)),
        toml::Value::Datetime(_) => Err("Datetime's are unsupported".into()),
        toml::Value::Array(ref a) => {
            let liquid_array: Result<Vec<liquid::Value>> = a.iter().map(convert_value).collect();
            let liquid_array = liquid_array?;
            Ok(liquid::Value::Array(liquid_array))
        }
        toml::Value::Table(ref t) => {
            let liquid_object: Result<collections::HashMap<String, liquid::Value>> = t.iter()
                .map(|(k, v)| {
                    let v = convert_value(v);
                    match v {
                        Ok(v) => Ok((k.to_string(), v)),
                        Err(e) => Err(e),
                    }
                })
                .collect();
            let liquid_object = liquid_object?;
            Ok(liquid::Value::Object(liquid_object))
        }
    }
}

fn build_context(path: &path::Path) -> Result<liquid::Context> {
    let mut input = String::new();
    let mut f = fs::File::open(path)?;
    f.read_to_string(&mut input)?;
    let input: toml::Value = input.parse()?;
    let value = convert_value(&input)?;
    let value = match value {
        liquid::Value::Object(o) => Ok(o),
        _ => Err("File must be a toml table"),
    }?;
    let data = liquid::Context::with_values(value);

    Ok(data)
}

fn run() -> Result<()> {
    let matches = clap::App::new("liquidate").version(crate_version!())
        .author(crate_authors!())
        .arg(option("input", "LIQUID").required(true))
        .arg(option("output", "TXT"))
        .arg(option("context", "TOML"))
        .arg(option("include-root", "PATH"))
        .get_matches_safe()?;

    let mut options = liquid::LiquidOptions::default();
    options.file_system = matches.value_of("include-root").map(path::PathBuf::from);

    let mut data = matches.value_of("context")
        .map(|s| {
            let p = path::PathBuf::from(s);
            build_context(p.as_path())
        })
        .map_or(Ok(None), |r| r.map(Some))?
        .unwrap_or_else(liquid::Context::new);

    let template_path =
        matches.value_of("input").map(path::PathBuf::from).expect("Parameter was required");
    let template = liquid::parse_file(template_path, options)?;
    let output = template.render(&mut data)?.unwrap_or_else(|| "".to_string());
    match matches.value_of("output") {
        Some(path) => {
            let mut out = fs::File::create(path::PathBuf::from(path))?;
            out.write_all(output.as_bytes())?;
        }
        None => {
            println!("{}", output);
        }
    }

    Ok(())
}

quick_main!(run);
