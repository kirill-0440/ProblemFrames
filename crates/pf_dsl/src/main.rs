use anyhow::Result;
use pf_dsl::validator::validate;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: pf_dsl <file.pf> [--dot | --report]");
        return Ok(());
    }

    let mode = if args.len() > 2 { &args[2] } else { "--dot" };
    let filename = &args[1];
    let path = std::path::Path::new(filename);

    // Use resolver to handle imports
    match pf_dsl::resolver::resolve(path, None) {
        Ok(problem) => match validate(&problem) {
            Ok(_) => {
                if mode == "--report" {
                    println!("{}", pf_dsl::report_gen::generate_report(&problem));
                } else if mode == "--gen-rust" {
                    match pf_dsl::codegen::generate_rust(&problem) {
                        Ok(code) => println!("{}", code),
                        Err(e) => eprintln!("Error generating code: {}", e),
                    }
                } else {
                    println!("{}", pf_dsl::dot_export::to_dot(&problem));
                }
            }
            Err(errors) => {
                eprintln!("Validation Errors:");
                for err in errors {
                    eprintln!("- {}", err);
                }
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
