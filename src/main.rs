mod assembler;
mod cpu;
mod lexer;
mod simulator;

use simulator::Simulator;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }
    let mut simulator = Simulator::new();

    if let Err(err) = simulator.run(&args[1]) {
        println!("Error: {:?}", err);
    }
}
