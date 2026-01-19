mod address;
mod assembler;
mod lexer;
mod registers;
mod simulator;

use simulator::Simulator;
use std::{env, process};

use crate::simulator::SimulatorError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeArgs {
    file: String,
    args: bool,
    help: bool,
    tokens: bool,
    instructions: bool,
    version: bool,
    memory: bool,
}

fn parse_args() -> RuntimeArgs {
    let args: Vec<String> = env::args().collect();
    let mut cli_args = RuntimeArgs::default();

    cli_args.file = match args.get(1) {
        Some(source) => source.to_string(),
        None => "".to_string(),
    };

    cli_args.help = args.contains(&"-h".to_string())
        || args.contains(&"--help".to_string())
        || cli_args.file.is_empty();

    cli_args.tokens = args.contains(&"-t".to_string()) || args.contains(&"--tokens".to_string());
    cli_args.args = args.contains(&"-a".to_string()) || args.contains(&"--args".to_string());
    cli_args.memory = args.contains(&"-m".to_string()) || args.contains(&"--memory".to_string());
    cli_args.version = args.contains(&"-v".to_string()) || args.contains(&"--version".to_string());
    cli_args.instructions =
        args.contains(&"-i".to_string()) || args.contains(&"--instructions".to_string());

    return cli_args;
}

fn main() {
    let args = parse_args();
    let package_name = env!("CARGO_PKG_NAME");
    let package_version = env!("CARGO_PKG_VERSION");

    if args.version {
        println!("{} v{}", package_name, package_version);
        return;
    }

    if args.help {
        println!("Usage: {} <file> [options]", package_name);
        println!("Options:");
        println!("  -h, --help     Print this help message");
        println!("  -a, --args     Print the arguments");
        println!("  -t, --tokens   Print the tokens");
        println!("  -i, --instructions   Print the instructions");
        println!("  -m, --memory   Print the memory");
        println!("  -v, --version  Print program version");
        return;
    }

    if args.args {
        println!("{:?}", args);
    }

    let mut assembler = assembler::Assembler::new();
    if let Err(err) = assembler.assemble(&args) {
        println!("Assembler Error: {:?}", err);
        return;
    }

    let memory = assembler.take_memory();

    if args.memory {
        println!("{:?}", memory);
    }

    let instructions = assembler.get_instructions();
    let entry = assembler.get_entry_point();

    let mut simulator = Simulator::new(instructions, memory, entry);

    let mut exit_code = 0;
    loop {
        if let Err(err) = simulator.step() {
            match err {
                SimulatorError::Exit(value) => {
                    exit_code = value as i32;
                    println!("\n-- program is finished running --");
                }
                SimulatorError::NoMoreInstructions => {
                    println!("\n-- program is finished running (dropped off bottom) --");
                }
                _ => println!("Simulator Error: {:?}", err),
            }
            break;
        }
    }
    process::exit(exit_code);
}
