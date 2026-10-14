use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("htpage {}", env!("CARGO_PKG_VERSION"));
        println!("High-performance markup language compiler powered by a pure Rust core\n");
        println!("USAGE:");
        println!("    htpage <file.htpage> [output.html]");
        println!("    htpage --version | -v");
        println!("    htpage --help | -h");
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    if args[1] == "--version" || args[1] == "-v" {
        println!("htpage {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let input_file = &args[1];
    let content = match fs::read_to_string(input_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{input_file}': {e}");
            process::exit(1);
        }
    };

    let compiled = match htpage_core::compile(&content) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Compilation error: {e}");
            process::exit(1);
        }
    };

    let output_file = if args.len() >= 3 {
        args[2].clone()
    } else if input_file.ends_with(".htpage") {
        format!("{}.html", &input_file[..input_file.len() - 7])
    } else {
        format!("{input_file}.html")
    };

    if let Err(e) = fs::write(&output_file, compiled) {
        eprintln!("Error writing output file '{output_file}': {e}");
        process::exit(1);
    }

    println!("Successfully compiled to {output_file}");
}
