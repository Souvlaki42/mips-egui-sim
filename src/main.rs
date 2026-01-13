mod assembler;
mod cpu;
mod tokenizer;

use cpu::Cpu;

fn main() {
    let cpu = Cpu::new();

    if let Err(err) = cpu.run("examples/hello_world.asm") {
        println!("Error: {:?}", err);
    }
}
