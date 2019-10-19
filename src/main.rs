use std::env::{self, var};
use std::process::exit;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;

static HELP_MESSAGE: &'static str = r#"Available commands:
    help                                    Show the help.
    add <list_name> <site_list>             Add sites to a proxy list. site_list is a comma separated list of urls or domains to add to the list.
    allow <list_name>                       Move all sites in that list to the "allowed" list
    cat <list_name> [pending|allowed]       Print out the given pending list or allowed list"#;

fn main() {
    let mut args = env::args();
    args.next(); // Ignore program path

    let subcommand;
    if let Some(cmd) = args.next() {
        subcommand = cmd;
    } else {
        eprintln!("You must specify a subcommand. Use \"help\" to see available commands.");
        exit(1);
    }

    match subcommand.as_str() {
        "help" => println!("{}", HELP_MESSAGE),
        "add" => subcommand_add(args),
        "allow" => subcommand_allow(args),
        "cat" => subcommand_cat(args),
        other_cmd => {
            eprintln!(r#"unrecognized command "{}""#, other_cmd);
            exit(1);
        }
    }
}

struct ListPaths {
    pub pending: String,
    pub allowed: String,
}

fn get_list_paths(args: &mut env::Args) -> ListPaths {
    let list_name;
    if let Some(name) = args.next() {
        list_name = name;
    } else {
        eprintln!("You must specify a list name. See help for details.");
        exit(1);
    }

    ListPaths {
        pending: format!("{}/proxylists/{}-pending.txt", var("HOME").unwrap(), list_name),
        allowed: format!("{}/proxylists/{}-allowed.txt", var("HOME").unwrap(), list_name),
    }
}

fn subcommand_add(mut args: env::Args) {
    let ListPaths { pending: list_path, allowed: _ } = get_list_paths(&mut args);

    let new_sites;
    if let Some(sites) = args.next() {
        new_sites = sites;
    } else {
        eprintln!("You must specify a list name. See help for details.");
        exit(1);
    }

    let new_sites = new_sites
        .split(",")
        .map(|item| item.trim())
        .filter(|&item| item != "")
        .map(|item| String::from(".") + &item.replace("http://", "").replace("https://", ""))
        .map(|item| String::from(item.split("/").next().unwrap()))
        .fold(String::new(), |prev_domains, next_domain| format!("{}\n{}", prev_domains, next_domain));
    
    let mut list_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&list_path)
        .expect(&format!("could not open proxy list: {}", &list_path));

    list_file.write_all(new_sites.as_bytes()).expect(&format!("Could not write to proxy list: {}", list_path));

    println!("Added pending: {}", &new_sites);
}

fn subcommand_allow(mut args: env::Args) {
    let list_paths = get_list_paths(&mut args);

    let mut allowed_list = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&list_paths.allowed)
        .expect(&format!("could not open proxy list: {}", &list_paths.allowed));

    let pending_list = read_to_string(&list_paths.pending).expect(&format!("Could not read proxy list: {}", list_paths.pending));

    allowed_list.write_all(pending_list.as_bytes()).expect(&format!("Could not write proxy list: {}", list_paths.allowed));

    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&list_paths.pending)
        .expect(&format!("could not open proxy list: {}", &list_paths.pending));

    println!("Added alowed: {}", &pending_list);
}

enum ListType {
    Pending,
    Allowed,
}

fn subcommand_cat(mut args: env::Args) {
    let list_paths = get_list_paths(&mut args);

    let mut list_type_arg = String::new();
    if let Some(list) = args.next() {
        list_type_arg = list
    }
    let list_type = match list_type_arg.as_str() {
        "allowed" => ListType::Allowed,
        "pending" => ListType::Pending,
        "" => ListType::Pending,
        x => {
            eprintln!("{} is not a valid list type", x);
            exit(1);
        }
    };

    match list_type {
        ListType::Pending => println!("{}", read_to_string(&list_paths.pending).expect(&format!("could not read proxy list: {}", list_paths.pending))),
        ListType::Allowed => println!("{}", read_to_string(&list_paths.allowed).expect(&format!("could not read proxy list: {}", list_paths.allowed))),
    }
}
