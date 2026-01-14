mod assembler;
mod lexer;
mod registers;
mod simulator;

use simulator::Simulator;
use std::{env, process};

use crate::simulator::SimulatorError;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }

    let mut assembler = assembler::Assembler::new();
    if let Err(err) = assembler.assemble(&args[1]) {
        println!("Assembler Error: {:?}", err);
        return;
    }

    let memory = assembler.take_memory();
    let instructions = assembler.get_instructions();
    let entry = assembler.get_entry_point();

    let mut simulator = Simulator::new(instructions, memory, entry);

    let mut exit_code = 0;
    loop {
        if let Err(err) = simulator.step() {
            match err {
                SimulatorError::NoMoreInstructions(value) => {
                    exit_code = value as i32;
                    println!("The execution has ended");
                }
                _ => println!("Error: {:?}", err),
            }
            break;
        }
    }
    process::exit(exit_code);
}
