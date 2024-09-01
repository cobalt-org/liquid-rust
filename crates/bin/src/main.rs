// Allow zero pointers for lazy_static. Otherwise clippy will complain.
#![allow(unknown_lints)]

use std::ffi;
use std::fs;
use std::io::Write;
use std::path;

use clap::Parser;

#[derive(Copy, Clone, Debug, derive_more::Display, derive_more::From, derive_more::Constructor)]
#[display("{}", msg)]
struct Error {
    msg: &'static str,
}

impl std::error::Error for Error {}

fn load_yaml(path: &path::Path) -> Result<liquid::Object, Box<dyn std::error::Error>> {
    let f = fs::File::open(path)?;
    serde_yaml::from_reader(f).map_err(|e| e.into())
}

fn load_json(path: &path::Path) -> Result<liquid::Object, Box<dyn std::error::Error>> {
    let f = fs::File::open(path)?;
    serde_json::from_reader(f).map_err(|e| e.into())
}

fn build_context(path: &path::Path) -> Result<liquid::Object, Box<dyn std::error::Error>> {
    let extension = path.extension().unwrap_or_else(|| ffi::OsStr::new(""));
    let value = match extension.to_str() {
        Some("yaml") => load_yaml(path),
        Some("json") => load_json(path),
        _ => Err(Error::new("Unsupported file type").into()),
    }?;

    Ok(value)
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: std::path::PathBuf,

    #[arg(long)]
    output: Option<std::path::PathBuf>,

    #[arg(long)]
    context: Option<std::path::PathBuf>,
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let args = Args::parse();

    let parser = liquid::ParserBuilder::with_stdlib()
        .build()
        .expect("should succeed without partials");
    let template = parser.parse_file(&args.input)?;

    let data = args
        .context
        .as_ref()
        .map(|p| build_context(p.as_path()))
        .map(|r| r.map(Some))
        .unwrap_or(Ok(None))?
        .unwrap_or_else(liquid::Object::new);
    let output = template.render(&data)?;
    match args.output {
        Some(path) => {
            let mut out = fs::File::create(path)?;
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
