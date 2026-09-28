use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: htpage <file.htpage> [output.html]");
        process::exit(1);
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
