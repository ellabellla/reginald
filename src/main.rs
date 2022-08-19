use std::{fs::File, process, fmt::Display, io::{Read, self}};

use clap::{Parser, ValueEnum};
use reginald_lib::regex::{Regex};

#[derive(Parser)]
struct Cli {
    #[clap(arg_enum)]
    command: Commands,
    regex:String,
    replace_str: Option<String>,
    #[clap(short, long, parse(from_os_str))]
    input: Option<std::path::PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Commands {
    MATCH,
    MATCHES,
    IS,
}

fn main() {
    let cli = Cli::parse();

    let regex = handle_error(Regex::compile(&cli.regex));
    let input =  match cli.input {
        Some(path) => {
            if !path.is_file() {
                println!("Error: \n\t Input must be a file.");
                return;
            }

            let mut file = handle_error(File::open(path));
            let mut input = String::new();
            handle_error(file.read_to_string(&mut input));
            input
        },
        None => {
            let mut stdin = io::stdin();
            let mut input = String::new();
            handle_error(stdin.read_to_string(&mut input));
            input
        },
    };

    let matches = match cli.command {
        Commands::MATCH => {
            match regex.is_match(&input) {
                Some(sub_str) => vec![sub_str],
                None => vec![],
            }
        },
        Commands::MATCHES => regex.matches(&input),
        Commands::IS => if !regex.test(&input) {vec![(0,input.len())]} else {vec![]},
    };
    
    let input = input.chars().collect::<Vec<char>>();
    match cli.replace_str {
        Some(replace_str) => {
            let mut matches = matches.iter().peekable();
            let mut i = 0;
            while i < input.len() {
                if let Some((start, size)) = matches.peek() {
                    if i < *start {
                        print!("{}", input[i..*start].iter().collect::<String>());
                        i = *start;
                    } else if *start == i {
                        print!("{}", replace_str);
                        i = *size + start;
                        matches.next();
                    } else {
                        unreachable!()
                    }
                } else {
                    print!("{}", input[i..input.len()].iter().collect::<String>());
                    break;
                }
            }
        },
        None => for (start, size) in matches {
            println!("{}", input[start..(start+size)].iter().collect::<String>())
        },
    }
}

fn handle_error<T,E>(result: Result<T,E>) -> T
where
    E: Display,
{
    match result {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error:\n\t{}", err);
            process::exit(1)
        },
    }
}