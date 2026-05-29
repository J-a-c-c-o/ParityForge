mod parity_game;
mod utils;
mod pg_parser;
mod solvers;

use clap::{Parser, Subcommand};
use crate::pg_parser::{parse_pg, strat_to_sol};
use crate::solvers::{run_fpi, run_tl, run_zielonka};

/// ParityTool CLI
#[derive(Parser)]
#[command(name = "parity-tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve using a specified algorithm
    Solve {
        /// Input file (.pg)
        input: String,

        /// Output file for the solution
        output: String,

        /// Algorithm name to use
        #[arg(long, default_value = "default")]
        algorithm: String,        
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Solve { algorithm, input, output } => {
            println!("Running `solve` with algorithm: {}", algorithm);
            match algorithm.as_str() {
                "default" => default(&input, &output),
                "zielonka" => zielonka(&input, &output),
                "fpi" => fpi(&input, &output),
                "tl" => tl(&input, &output),
                _ => {
                    eprintln!("Algorithm '{}' not implemented yet.", algorithm);
                    std::process::exit(2);
                }
            }
        }
    }
}

fn default(input: &str, output_file: &str) {
    zielonka(input, output_file);
}


fn zielonka(file: &str, output_file: &str) {
    let input = std::fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", file, e);
        std::process::exit(1);
    });

    let game = parse_pg(&input);
    let result = run_zielonka(&game.clone().unwrap());
    if let Err(e) = result {
        eprintln!("Error running Zielonka algorithm: {}", e);
        std::process::exit(1);
    }

    if let Ok((winning_region0, winning_region1, strategy_0, strategy_1)) = result {
        let output = strat_to_sol(&game.unwrap(), &strategy_0, &strategy_1, &winning_region0, &winning_region1);

        std::fs::write(&output_file, output).unwrap_or_else(|e| {
            eprintln!("Error writing output file '{}': {}", output_file, e);
            std::process::exit(1);
        });
        println!("Solution written to '{}'", output_file);
    }

    
}

fn fpi(file: &str, output_file: &str) {
    let input = std::fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", file, e);
        std::process::exit(1);
    });

    let game = parse_pg(&input);
    let result = run_fpi(&game.clone().unwrap());
    if let Err(e) = result {
        eprintln!("Error running FPI algorithm: {}", e);
        std::process::exit(1);
    }

    if let Ok((winning_region0, winning_region1, strategy_0, strategy_1)) = result {
        let output = strat_to_sol(&game.unwrap(), &strategy_0, &strategy_1, &winning_region0, &winning_region1);

        std::fs::write(&output_file, output).unwrap_or_else(|e| {
            eprintln!("Error writing output file '{}': {}", output_file, e);
            std::process::exit(1);
        });
        println!("Solution written to '{}'", output_file);
    }
}

fn tl(file: &str, output_file: &str) {
    let input = std::fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", file, e);
        std::process::exit(1);
    });

    let game = parse_pg(&input);
    let result = run_tl(&game.clone().unwrap());
    if let Err(e) = result {
        eprintln!("Error running tangle learning algorithm: {}", e);
        std::process::exit(1);
    }

    if let Ok((winning_region0, winning_region1, strategy_0, strategy_1)) = result {
        let output = strat_to_sol(&game.unwrap(), &strategy_0, &strategy_1, &winning_region0, &winning_region1);

        std::fs::write(&output_file, output).unwrap_or_else(|e| {
            eprintln!("Error writing output file '{}': {}", output_file, e);
            std::process::exit(1);
        });
        println!("Solution written to '{}'", output_file);
    }
}



