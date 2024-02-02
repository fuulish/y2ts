use std::env;
use std::fs;
use std::process;

fn parse(mut args: env::Args) -> Result<String, &'static str> {
    args.next(); // get rid of program name

    let source = match args.next() {
        Some(arg) => arg,
        None => return Err("no source grammar"),
    };

    Ok(source)
}

fn main() {
    let source = parse(env::args()).unwrap_or_else(|err| {
        eprintln!("Error parsing {}", err);
        process::exit(1);
    });

    let content = fs::read_to_string(source);
}
