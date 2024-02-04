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

    for rule in content.split(";\n") {
        // XXX: <-- this is not the original logic (b/c that didn't
        // work for languages with semi-colons in rule actions)
        // XXX: generalize using regular expressions!
        if rule.is_empty() {
            continue;
        }
        let mut split = rule.trim().split(":");

        let name = split.next().unwrap(); // This really should not fail

        let branches: Vec<&str> = split.next().unwrap().split("|").collect(); // this shouldn't fail either

        let formed_rule = match branches.len() {
            0 => continue,
            1 => from_one_branch_rule(name, branches),
            _ => from_many_branches_rule(name, branches),
        };

        output.push_str(&formed_rule);
    }

    output.push_str("}\n});");
}

fn from_many_branches_rule(rule_name: &str, branch_rule: Vec<&str>) -> String {
    branch_rule[0].to_owned()
}

fn from_one_branch_rule(rule_name: &str, rule_branches: Vec<&str>) -> String {
    let mut builder = String::new();
    builder.push_str(&format!("{rule_name}: $ => "));

    let branch = rule_branches[0].trim().split(" ").collect::<Vec<_>>();
    if branch.len() == 1 {
        builder.push_str(&format!("$.{},", branch[0]));
    } else {
        builder.push_str(process_branch(branch))
    }
    builder.push_str("\n");
    builder
}

fn process_branch(branch: Vec<&str>) -> &str {
    branch[0]
}

fn remove_semantic_actions(rule: &str) -> &str {
    rule
}
