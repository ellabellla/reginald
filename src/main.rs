use std::{fs::File, process, fmt::Display, io::{Read, self}};

use clap::{Parser, ValueEnum};
use reginald_lib::regex::{Regex};

#[derive(Parser)]
struct Cli {
    regex:String,
    #[clap(arg_enum, default_value_t = Commands::MATCHES)]
    command: Commands,
    #[clap(parse(from_os_str))]
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

            let mut file = handle_error(File::create(path));
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
    for (start, size) in matches {
        println!("{}", input[start..(start+size)].iter().collect::<String>())
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