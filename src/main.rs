mod parity_game;
mod pg_parser;
mod solvers;
mod verifier;

use clap::{Parser, Subcommand};
use crate::pg_parser::{parse_pg, sol_to_strat, strat_to_sol};
use crate::solvers::{run_fpi, run_tl, run_zielonka, run_spm, run_si};
use crate::parity_game::{ParityGame, ParityGameBuilder};
use crate::verifier::verify_solution;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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

    /// Run one or more algorithms over every .pg file in a file or folder
    Test {
        /// Input file or directory containing .pg files
        input: Option<String>,

        /// Count of random games to generate and test; ignored if input is a file or directory
        #[arg(long= "count")]
        random_count: Option<usize>,

        /// Number of nodes for random games; ignored if input is a file or directory
        #[arg(long = "size")]
        random_nodes: Option<usize>,

        /// Maximum number of edges for a node for random games; ignored if input is a file or directory
        #[arg(long = "maxe")]
        max_edges: Option<usize>,

        /// Maximum priority for random games; ignored if input is a file or directory
        #[arg(long = "maxp")]
        max_prio: Option<usize>,

        /// Seed for random game generation; ignored if input is a file or directory
        #[arg(long = "seed")]
        seed: Option<u64>,

        /// Algorithm name to use; repeat this flag to test multiple algorithms
        #[arg(long = "alg")]
        algorithms: Vec<String>,
    },

    /// Verify a solution file against a game file
    Verify {
        /// Input game file (.pg)
        game: String,
        /// Input solution file (.paritysol)
        solution: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Solve { algorithm, input, output } => {
            run_solve_command(&input, &output, &algorithm);
        }

        Commands::Test { input, random_count, random_nodes, max_edges, max_prio, algorithms, seed } => {
            run_test_command(input, random_count, random_nodes, max_edges, max_prio, algorithms, seed);
        }

        Commands::Verify { game, solution } => {
            run_verify_command(game, solution);
        }
        
    }
}

fn run_solve_command(input: &str, output: &str, algorithm: &str) {
    match algorithm {
        "default" | "zlk" => zielonka(input, output),
        "fpi" => fpi(input, output),
        "tl" => tl(input, output),
        "spm" => spm(input, output),
        "si" => si(input, output),
        _ => {
            eprintln!("Algorithm '{}' not implemented yet.", algorithm);
            std::process::exit(2);
        }
    }
}

fn run_test_command(input: Option<String>, random_count: Option<usize>, random_nodes: Option<usize>, max_edges: Option<usize>, max_prio: Option<usize>, algorithms: Vec<String>, seed: Option<u64>) {
    let algorithms = if algorithms.is_empty() {
            vec![
            String::from("zlk"),
            String::from("fpi"),
            String::from("tl"),
            String::from("spm"),
            String::from("si"),
        ]
    } else {
        algorithms
    };

    let mut combined_times: HashMap<String, std::time::Duration> = HashMap::new();
    let mut failures = 0usize;

    if let Some(input) = input {
        let input_paths = collect_pg_inputs(Path::new(&input)).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

        if input_paths.is_empty() {
            eprintln!("No .pg files found under '{}'", input);
            std::process::exit(1);
        }

        
        for path in input_paths {
            let input_text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Error reading file '{}': {}", path.display(), e);
                std::process::exit(1);
            });

            let game = parse_pg(&input_text).unwrap_or_else(|e| {
                eprintln!("Error parsing parity game '{}': {}", path.display(), e);
                std::process::exit(1);
            });

            for algorithm in &algorithms {
                let start_time = std::time::Instant::now();
                let (w0, w1, strat0, strat1) = solve_game(&game, algorithm);
                let duration = start_time.elapsed();
                combined_times.entry(algorithm.clone()).and_modify(|d| *d += duration).or_insert(duration);
                match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
                    Ok(()) => {
                        println!("[ok] {} via {} in {:.2?}", path.display(), algorithm, duration);
                    }
                    Err(e) => {
                        failures += 1;
                        eprintln!("[fail] {} via {}: {} in {:.2?}", path.display(), algorithm, e, duration);
                    }
                }
            }
        }
    } else {
        let count = random_count.unwrap_or(100);
        let nodes = random_nodes.unwrap_or(100);
        let max_edges = max_edges.unwrap_or(4);
        let max_prio = max_prio.unwrap_or(nodes);

        for i in 0..count {
            let game = ParityGameBuilder::new().random_game(nodes, max_edges, max_prio, seed).build();
            for algorithm in &algorithms {
                let start_time = std::time::Instant::now();
                let (w0, w1, strat0, strat1) = solve_game(&game, algorithm);
                let duration = start_time.elapsed();
                combined_times.entry(algorithm.clone()).and_modify(|d| *d += duration).or_insert(duration);
                match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
                    Ok(()) => {
                        println!("[ok] random game #{} via {} in {:.2?}", i + 1, algorithm, duration);
                    }
                    Err(e) => {
                        failures += 1;
                        eprintln!("[fail] random game #{} via {}: {} in {:.2?}", i + 1, algorithm, e, duration);
                    }
                }
            }
        }

    }

    println!("Combined times:");
    for (algorithm, duration) in combined_times {
        println!("  {}: {:.2?}", algorithm, duration);
    }

    if failures > 0 {
        eprintln!("{} test run(s) failed", failures);
        
        std::process::exit(1);
    }

    println!("All requested algorithm runs passed.");
}

fn run_verify_command(game_file: String, solution_file: String) {
    let game_input = std::fs::read_to_string(&game_file).unwrap_or_else(|e| {
        eprintln!("Error reading game file '{}': {}", game_file, e);
        std::process::exit(1);
    });

    let solution_input = std::fs::read_to_string(&solution_file).unwrap_or_else(|e| {
        eprintln!("Error reading solution file '{}': {}", solution_file, e);
        std::process::exit(1);
    });

    let game = parse_pg(&game_input).unwrap_or_else(|e| {
        eprintln!("Error parsing parity game '{}': {}", game_file, e);
        std::process::exit(1);
    });

    let (w0, w1, strat0, strat1) = sol_to_strat(&solution_input).unwrap_or_else(|e| {
        eprintln!("Error parsing solution file '{}': {}", solution_file, e);
        std::process::exit(1);
    });

    match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
        Ok(()) => println!("Solution in '{}' is valid for game '{}'", solution_file, game_file),
        Err(e) => {
            eprintln!("Solution in '{}' is invalid for game '{}': {}", solution_file, game_file, e);
            std::process::exit(1);
        }
    }
}


fn run_algorithm<Algo>(input_file: &str, output_file: &str, algorithm: Algo, alg_name: &str)
    where Algo: Fn(&ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String>
{
    let input = std::fs::read_to_string(input_file).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", input_file, e);
        std::process::exit(1);
    });

    let game = parse_pg(&input).unwrap_or_else(|e| {
        eprintln!("Error parsing parity game: {}", e);
        std::process::exit(1);
    });

    let result = algorithm(&game);
    if let Err(e) = result {
        eprintln!("Error running {} algorithm: {}", alg_name, e);
        std::process::exit(1);
    }

    if let Ok((winning_region0, winning_region1, strategy_0, strategy_1)) = result {
        let output = strat_to_sol(&game, &strategy_0, &strategy_1, &winning_region0, &winning_region1);

        std::fs::write(output_file, output).unwrap_or_else(|e| {
            eprintln!("Error writing output file '{}': {}", output_file, e);
            std::process::exit(1);
        });
        println!("Solution written to '{}'", output_file);
    }
}

fn zielonka(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_zielonka, "Zielonka's Recursive Algorithm");
}

fn fpi(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_fpi, "Fixed-Point Iteration Algorithm");
}

fn tl(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_tl, "Tangle Learning Algorithm");
}

fn spm(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_spm, "Small Progress Measures Algorithm");
}

fn si(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_si, "Strategy Improvement Algorithm");
}

fn solve_game(
    game: &ParityGame,
    algorithm: &str,
) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    match algorithm {
        "default" | "zlk" => run_zielonka(game).unwrap_or_else(|e| exit_algorithm_error("Zielonka's algorithm", &e)),
        "fpi" => run_fpi(game).unwrap_or_else(|e| exit_algorithm_error("FPI algorithm", &e)),
        "tl" => run_tl(game).unwrap_or_else(|e| exit_algorithm_error("Tangle Learning algorithm", &e)),
        "spm" => run_spm(game).unwrap_or_else(|e| exit_algorithm_error("SPM algorithm", &e)),
        "si" => run_si(game).unwrap_or_else(|e| exit_algorithm_error("SI algorithm", &e)),
        _ => {
            eprintln!("Algorithm '{}' not implemented yet.", algorithm);
            std::process::exit(2);
        }
    }
}

fn exit_algorithm_error(algorithm_name: &str, error: &str) -> ! {
    eprintln!("Error running {}: {}", algorithm_name, error);
    std::process::exit(1);
}

fn collect_pg_inputs(path: &Path) -> Result<Vec<PathBuf>, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("Error reading '{}': {}", path.display(), e))?;
    let mut files = Vec::new();

    if metadata.is_file() {
        if is_pg_file(path) {
            files.push(path.to_path_buf());
        }
        return Ok(files);
    }

    if metadata.is_dir() {
        collect_pg_files_recursive(path, &mut files)?;
        files.sort();
        return Ok(files);
    }

    Err(format!("'{}' is neither a file nor a directory", path.display()))
}

fn collect_pg_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("Error reading directory '{}': {}", dir.display(), e))? {
        let entry = entry.map_err(|e| format!("Error reading directory entry in '{}': {}", dir.display(), e))?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|e| format!("Error reading metadata for '{}': {}", path.display(), e))?;

        if metadata.is_dir() {
            collect_pg_files_recursive(&path, files)?;
        } else if metadata.is_file() && is_pg_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_pg_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("pg"))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity_game::ParityGameBuilder;


    fn example_game() -> ParityGame {
        let mut builder = ParityGameBuilder::new();
        let builder = builder
            .add_edge(0, 4)
            .add_edge(4, 0)
            .add_edge(7, 4)
            .add_edge(1, 7)
            .add_edge(4, 8)
            .add_edge(8, 6)
            .add_edge(7, 6)
            .add_edge(2, 1)
            .add_edge(2, 3)
            .add_edge(3, 2)
            .add_edge(3, 5)
            .add_edge(5, 3)
            .add_edge(5, 1)
            .add_edge(1, 5)
            .add_edge(6, 2)
            .set_owner(0, 0)
            .set_owner(1, 1)
            .set_owner(2, 0)
            .set_owner(3, 0)
            .set_owner(4, 1)
            .set_owner(5, 0)
            .set_owner(6, 0)
            .set_owner(7, 0)
            .set_owner(8, 0)
            .set_priority(0, 0)
            .set_priority(1, 1)
            .set_priority(2, 2)
            .set_priority(3, 3)
            .set_priority(4, 2)
            .set_priority(5, 5)
            .set_priority(6, 6)
            .set_priority(7, 7)
            .set_priority(8, 8);
        
        let game = builder.build();
        game
    }


    #[test]
    fn test_solve_si() {
        let game = example_game();
        let (w0, w1, strat0, strat1) = run_si(&game).unwrap();
        println!("Winning set for player 0: {:?}", w0);
        println!("Winning set for player 1: {:?}", w1);
        println!("Strategy for player 0: {:?}", strat0);
        println!("Strategy for player 1: {:?}", strat1);

        // validate the solution
        match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
            Ok(()) => println!("SI solution is valid"),
            Err(e) => panic!("SI solution is invalid: {}", e),
        }
        
        // print to file /tmp/si_solution.txt for debugging
        let output = strat_to_sol(&game, &strat0, &strat1, &w0, &w1);
        std::fs::write("./tmp/si_solution.txt", output).unwrap();
    }

    

}