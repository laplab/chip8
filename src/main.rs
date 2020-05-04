use chip8::{Emulator, EmulatorError};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path to CHIP-8 ROM>", args[0]);
        std::process::exit(1);
    }

    let mut emulator = Emulator::new();
    if let Err(e) = emulator.run(&args[1]) {
        match e {
            EmulatorError::ReadProgramError(m) => eprintln!("Error reading ROM: {}", m),
            EmulatorError::RuntimeError(m) => eprintln!("Error executing ROM: {}", m),
        }
    }
}
