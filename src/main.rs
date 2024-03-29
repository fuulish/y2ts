use std::env;
use std::fs;
use std::io::prelude::*;
use std::process;

use regex::Regex;

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

    let mut optionals = Vec::<String>::new();
    let content = &content[rules_start + 2..rules_end];
    let content = cleanup_grammar(content.trim());
    // XXX: also remove tokens, terminals, and non-terminals, here, and everything else that
    // doesn't belong

    let mut buffer = fs::File::create("cleaned_grammar.y").unwrap();
    buffer.write_all(&content.as_bytes()).unwrap();

    let mut output = String::new();
    output.push_str(
        "module.exports = grammar({\n\
             name: 'ALANG',\n\
             rules: {",
    );

    for rule in content.split(";") {
        // XXX: <-- this is not the original logic (b/c that didn't
        // work for languages with semi-colons in rule actions)
        // XXX: generalize using regular expressions!
        if rule.trim().is_empty() {
            continue;
        }

        let mut split = rule.trim().split(":");

        let name = split.next().unwrap(); // This really should not fail

        // XXX: still need to cleanup rule processing
        //      there are items (e.g. tokens, non-terminals and the like declared using % at the
        //      beginning of lines)
        // XXX: what about %empty rules?
        let branches: Vec<&str> = split.next().unwrap().split("|").collect(); // this shouldn't fail either

        let formed_rule = match branches.len() {
            0 => continue,
            1 => from_one_branch_rule(name, branches),
            _ => from_many_branches_rule(name, branches, &mut optionals),
        };

        output.push_str(&formed_rule);
    }

    output.push_str("}\n});");
    output = post_process(output, &optionals);
    println!("{output}");
}

// XXX: this could probably be handled better, and directly included in the from_many_branches_rule
fn post_process(mut output: String, optionals: &Vec<String>) -> String {
    for optional_rule in optionals.iter() {
        output = output.replace(
            &format!("$.{},", optional_rule),
            &format!("optional($.{}),", optional_rule),
        );
    }

    output
}

fn make_header(name: &str) -> String {
    format!("{name}: $ => ")
}

fn from_many_branches_rule(
    rule_name: &str,
    branch_rule: Vec<&str>,
    optionals: &mut Vec<String>,
) -> String {
    let mut builder = String::new();
    builder.push_str(&make_header(rule_name));

    let actually_more_than_one_branch = branch_rule.iter().filter(|&x| !x.is_empty()).count() > 1;
    if actually_more_than_one_branch {
        builder.push_str("choice(\n");
    }

    for branch in branch_rule.iter() {
        if branch.trim().is_empty() {
            optionals.push(rule_name.to_owned()); // XXX:say what???
        } else {
            builder.push_str(&process_branch(branch.trim().split_whitespace().collect()));
        }
    }

    if actually_more_than_one_branch {
        builder.push_str("),\n\n");
    }

    builder
}

fn get_token(data: &str) -> String {
    if data.starts_with("'") && data.ends_with("'") {
        data.to_owned()
    } else {
        format!("$.{}", data)
    }
}

fn from_one_branch_rule(rule_name: &str, rule_branches: Vec<&str>) -> String {
    let mut builder = String::new();
    builder.push_str(&make_header(rule_name));

    let branch = rule_branches[0]
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>();
    if branch.len() == 1 {
        builder.push_str(&get_token(&branch[0]));
    } else {
        builder.push_str(&process_branch(branch))
    }
    builder.push_str("\n");
    builder
}

fn process_branch(branch: Vec<&str>) -> String {
    let mut builder = String::new();

    if branch.len() > 1 {
        builder.push_str("seq(\n");
    }
    for element in branch.iter() {
        builder.push_str(&format!("{},\n", get_token(element)));
    }
    if branch.len() > 1 {
        builder.push_str("),\n");
    }
    builder
}

fn remove_first_semantic_action(content: &str) -> Option<String> {
    let mut first_brace = match content.find("{") {
        Some(i) => i,
        None => return None,
    };

    let mut open_part = &content[first_brace + 1..];
    let last_brace;

    let mut stripped = content[..first_brace].to_owned();

    let mut nopen = 1;

    first_brace += 1;
    loop {
        let open_brace = match open_part.find("{") {
            Some(i) => i,
            None => open_part.len(),
        };
        let close_brace = match open_part.find("}") {
            Some(i) => i,
            None => open_part.len(),
        };

        nopen = if close_brace < open_brace {
            first_brace += close_brace;

            nopen - 1
        } else if open_brace < close_brace {
            first_brace += open_brace;

            nopen + 1
        } else {
            panic!();
        };
        first_brace += 1;

        if nopen == 0 {
            last_brace = first_brace;
            break;
        }

        open_part = &content[first_brace..];
    }

    stripped.push_str(&content[last_brace..]);
    Some(stripped)
}

fn remove_semantic_actions(content: &str) -> String {
    let mut copy = content.to_owned();
    while let Some(value) = remove_first_semantic_action(&copy) {
        copy = value;
    }
    copy
}

fn cleanup_grammar(content: &str) -> String {
    // let semantic_actions_regex = Regex::new("\\{(.|\\n)+?}").unwrap();
    let comments_regex = Regex::new("(//.*?\\n|/\\*(.|\\n)*?\\*/)").unwrap();
    let tokens_terminals_regex = Regex::new("\\n(%nterm|%token).+?;").unwrap();
    let no_actions = remove_semantic_actions(&content);
    let semicolon_apostrophe = Regex::new("';'").unwrap(); // XXX: just a hack

    // let result = remove_semantic_actions(&rule);
    let result = comments_regex.replace_all(&no_actions, "");
    let result = semicolon_apostrophe.replace_all(&result, "SEMICOLON"); // XXX: just a hack
    tokens_terminals_regex
        .replace_all(&result, "")
        .as_ref()
        .to_owned()
}
