use clap::{Parser, Subcommand};
use solver::{load_pg, save_solution, solve, Algorithm};
use std::path::Path;

/// ParityForge CLI
#[derive(Parser)]
#[command(
    name = "parityforge",
    version,
    about = "A tool for solving parity games with various algorithms"
)]
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
        #[arg(long = "algo", short = 'a', default_value = "default")]
        algorithm: String,
    },

    /// Verify a solution file against a game file
    Verify {
        /// Input game file (.pg)
        game: String,
        /// Input solution file (.paritysol)
        solution: String,
    },

    /// List available algorithms
    ListAlgorithms,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Solve { input, output, algorithm } => {
            let algo = algorithm.parse::<Algorithm>().unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });

            let game = load_pg(Path::new(&input)).unwrap_or_else(|e| {
                eprintln!("Failed to load game: {}", e);
                std::process::exit(1);
            });

            let solution = solve(&game, algo).unwrap_or_else(|e| {
                eprintln!("Failed to solve game: {}", e);
                std::process::exit(1);
            });

            save_solution(Path::new(&output), &game, &solution).unwrap_or_else(|e| {
                eprintln!("Failed to save solution: {}", e);
                std::process::exit(1);
            });

            println!("Successfully solved and saved to {}", output);
        }
        Commands::Verify { game, solution } => {
            // Similarly thin: load_pg -> load_solution -> verify
            let game = load_pg(Path::new(&game)).unwrap_or_else(|e| {
                eprintln!("Failed to load game: {}", e);
                std::process::exit(1);
            });

            let solution = solver::load_solution(Path::new(&solution)).unwrap_or_else(|e| {
                eprintln!("Failed to load solution: {}", e);
                std::process::exit(1);
            });

            solver::verify(&game, &solution).unwrap_or_else(|e| {
                eprintln!("Verification failed: {}", e);
                std::process::exit(1);
            });

            println!("Solution verified successfully.");
        }
        Commands::ListAlgorithms => {
            solver::Algorithm::list_algorithms().iter().for_each(|algo| {
                println!("{}", algo);
            });
        }
    }
}