mod parity_game;
mod utils;
mod pg_parser;
// mod tangle_learning;
mod zielonka;

use clap::{Parser, Subcommand};
use crate::pg_parser::parse_pg;
// use crate::tangle_learning::run_tl;
use crate::zielonka::run_zielonka;

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
        /// Input file
        input: String,

        /// Algorithm name to use
        #[arg(long, default_value = "default")]
        algorithm: String,        
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Solve { algorithm, input } => {
            println!("Running `solve` with algorithm: {}", algorithm);
            match algorithm.as_str() {
                "default" => default(&input),
                // "tl" => tl(&input),
                "zielonka" => zielonka(&input),
                _ => {
                    eprintln!("Algorithm '{}' not implemented yet.", algorithm);
                    std::process::exit(2);
                }
            }
        }
    }
}

fn default(input: &str) {
    zielonka(input);
}

// fn tl(input: &str) {
//     let game = parse_pg(input);
//     let result = run_tl(&game.unwrap());
//     if let Err(e) = result {
//         eprintln!("Error running TL algorithm: {}", e);
//         std::process::exit(1);
//     }
// }

fn zielonka(input: &str) {
    let game = parse_pg(input);
    let result = run_zielonka(&game.unwrap());
    if let Err(e) = result {
        eprintln!("Error running Zielonka algorithm: {}", e);
        std::process::exit(1);
    }
}



