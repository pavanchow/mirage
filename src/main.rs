use clap::{Parser, Subcommand};
use mirage::{assemble, decode, encode, Vm};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mirage", about = "A small stack-based bytecode virtual machine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Assemble and run a .asm program directly.
    Run { path: PathBuf },
    /// Assemble a .asm program into a .mbc bytecode file.
    Asm {
        path: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Disassemble a .mbc bytecode file back into readable instructions.
    Disasm { path: PathBuf },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {}", msg);
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Run { path } => {
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let program = assemble(&source).map_err(|e| e.to_string())?;
            let mut vm = Vm::new();
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            vm.run(&program, &mut handle).map_err(|e| e.to_string())
        }
        Command::Asm { path, output } => {
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let program = assemble(&source).map_err(|e| e.to_string())?;
            let bytes = encode(&program);
            fs::write(&output, bytes).map_err(|e| e.to_string())
        }
        Command::Disasm { path } => {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let program = decode(&bytes).map_err(|e| e.to_string())?;
            for (i, op) in program.ops.iter().enumerate() {
                println!("{:04}: {}", i, op);
            }
            Ok(())
        }
    }
}
