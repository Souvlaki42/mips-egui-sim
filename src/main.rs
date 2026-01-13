mod assembler;
mod cpu;
mod tokenizer;

use cpu::Cpu;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }
    let cpu = Cpu::new();

    if let Err(err) = cpu.run(&args[1]) {
        println!("Error: {:?}", err);
    }
}
