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

    let rules_start = content.find("%%").unwrap_or_else(|| {
        eprintln!("Could not find start of grammar rules");
        process::exit(1);
    });

    let rules_end = content[rules_start + 2..].find("%%").unwrap_or_else(|| {
        eprintln!("Could not find end of grammar rules");
        process::exit(1);
    }) + rules_start
        + 2;

    println!("{rules_start}:{rules_end}");

    let content = &content[rules_start + 2..rules_end];
    let content = remove_semantic_actions(content.trim());

    let mut output = String::new();
    output.push_str(
        "module.exports = grammar({\n\
             name: 'ALANG',\n\
             \n
             rules: {",
    );

    let rules = content
        .split(";")
        .filter(|rule| !rule.is_empty())
        .filter(|rule| {
            let mut rule_parts = rule.split(":");
            let rule_name = rule_parts.next().unwrap();
            let rule_branches = rule_parts.next().unwrap(); // XXX: could be rightfully empty, do not handle
                                                            // with panic

            let mut rule_iter = rule_branches.split("|");

            !rule_iter.next().unwrap().is_empty()

            /*
            rule_branches
                .split("|")
                .filter(|&branch| !branch.is_empty())
                .map(|branch| from_branch_rule(branch))
                .collect::<Vec<_>>()
            */
        })
        .map(|rule| {
            println!("{rule}");
            rule
        })
        .collect::<Vec<_>>();

    for line in rules {
        output.push_str(line);
    }

    output.push_str("}\n});");
}

fn from_branch_rule(branch_rule: &str) -> &str {
    branch_rule
}

fn remove_semantic_actions(rule: &str) -> &str {
    rule
}
