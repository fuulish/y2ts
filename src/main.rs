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

    let content = fs::read_to_string(source).unwrap_or_else(|err| {
        eprintln!("Error reading from file {}", err);
        process::exit(1);
    });

    let rules_start = match content.find("%%") {
        Some(index) => index,
        None => {
            eprintln!("Could not find start of grammar rules");
            process::exit(1);
        }
    };

    let rules_end = match content[rules_start..].find("%%") {
        Some(index) => index,
        None => {
            eprintln!("Could not find end of grammar rules");
            process::exit(1);
        }
    };

    let content = &content[rules_start + 2..rules_end];
    let content = remove_semantic_actions(content.trim());

    let mut output = String::new();
    output.push_str(
        "module.exports = grammar({\n\
             name: 'ALANG',\n\
             \n
             rules: {",
    );

    output.push_str("}\n});");
}

fn remove_semantic_actions(rule: &str) -> &str {
    rule
}
