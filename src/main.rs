mod parity_game;
mod pg_parser;
mod solvers;
mod verifier;

use crate::parity_game::{ParityGame, ParityGameBuilder};
use crate::pg_parser::{parse_pg, sol_to_strat, strat_to_sol, unparse_pg};
use crate::solvers::{run_external_solver, run_fpi, run_si, run_spm, run_tl, run_zielonka, run_unoptimized_zielonka};
use crate::verifier::verify_solution;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// ParityForge CLI
#[derive(Parser)]
#[command(
    name = "parity-forge",
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

    /// Run one or more algorithms over every .pg file in a file or folder
    Test {
        /// Input file or directory containing .pg files
        input: Option<String>,

        /// Count of random games to generate and test; ignored if input is a file or directory
        #[arg(long = "count", short = 'c')]
        random_count: Option<usize>,

        /// Number of nodes for random games; ignored if input is a file or directory
        #[arg(long = "size", short = 's')]
        random_nodes: Option<usize>,

        /// Maximum number of edges for a node for random games; ignored if input is a file or directory
        #[arg(long = "maxe", short = 'e')]
        max_edges: Option<usize>,

        /// Maximum priority for random games; ignored if input is a file or directory
        #[arg(long = "maxp", short = 'p')]
        max_prio: Option<usize>,

        /// Seed for random game generation; ignored if input is a file or directory
        #[arg(long = "seed", short = 'd')]
        seed: Option<u64>,

        /// Algorithm name to use; repeat this flag to test multiple algorithms
        #[arg(long = "algo", short = 'a')]
        algorithms: Vec<String>,

        /// External command to run as an algorithm, using %I for input file and %O for output file; repeat this flag to test multiple external algorithms
        #[arg(long = "external", short = 'x')]
        external_commands: Vec<String>,
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
        Commands::Solve {
            algorithm,
            input,
            output,
        } => {
            run_solve_command(&input, &output, &algorithm);
        }

        Commands::Test {
            input,
            random_count,
            random_nodes,
            max_edges,
            max_prio,
            algorithms,
            seed,
            external_commands,
        } => {
            run_test_command(
                input,
                random_count,
                random_nodes,
                max_edges,
                max_prio,
                algorithms,
                seed,
                external_commands,
            );
        }

        Commands::Verify { game, solution } => {
            run_verify_command(game, solution);
        }
    }
}

fn run_solve_command(input: &str, output: &str, algorithm: &str) {
    match algorithm {
        "default" | "zlk" => zielonka(input, output),
        "uzlk" => unoptimized_zielonka(input, output),
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

fn run_test_command(
    input: Option<String>,
    random_count: Option<usize>,
    random_nodes: Option<usize>,
    max_edges: Option<usize>,
    max_prio: Option<usize>,
    algorithms: Vec<String>,
    seed: Option<u64>,
    external_commands: Vec<String>,
) {
    let algorithms = if algorithms.is_empty() && external_commands.is_empty() {
        vec![
            String::from("zlk"),
            String::from("uzlk"),
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
                let result = solve_game(&game, algorithm);
                let duration = start_time.elapsed();
                combined_times
                    .entry(algorithm.clone())
                    .and_modify(|d| *d += duration)
                    .or_insert(duration);
                let (w0, w1, strat0, strat1) = match result {
                    Ok(res) => res,
                    Err(e) => {
                        failures += 1;
                        eprintln!(
                            "[fail] {} via {}: Error running algorithm: {} in {:.2?}",
                            path.display(),
                            algorithm,
                            e,
                            duration
                        );
                        continue;
                    }
                };
                match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
                    Ok(()) => {
                        println!(
                            "[ok] {} (size: {},{}) via {} in {:.2?}",
                            path.display(),
                            game.num_nodes(),
                            game.num_edges(),
                            algorithm,
                            duration
                        );
                    }
                    Err(e) => {
                        failures += 1;
                        eprintln!(
                            "[fail] {} via {}: {} in {:.2?}",
                            path.display(),
                            algorithm,
                            e,
                            duration
                        );
                    }
                }
            }

            for external_command in &external_commands {
                let output_file = std::env::temp_dir().join(format!(
                    "parity-forge-test-{}.paritysol",
                    path.file_stem().unwrap().to_string_lossy()
                ));
                let start_time = std::time::Instant::now();
                let result = run_external_solver(external_command, &path, &output_file);
                let duration = start_time.elapsed();
                combined_times
                    .entry(external_command.clone())
                    .and_modify(|d| *d += duration)
                    .or_insert(duration);
                match result {
                    Ok((w0, w1, strat0, strat1)) => {
                        match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
                            Ok(()) => {
                                println!(
                                    "[ok] {} (size: {},{}) via external '{}' in {:.2?}",
                                    path.display(),
                                    game.num_nodes(),
                                    game.num_edges(),
                                    external_command,
                                    duration
                                );
                            }
                            Err(e) => {
                                failures += 1;
                                eprintln!(
                                    "[fail] {} (size: {},{}) via external '{}': {} in {:.2?}",
                                    path.display(),
                                    game.num_nodes(),
                                    game.num_edges(),
                                    external_command,
                                    e,
                                    duration
                                );
                            }
                        }
                    }
                    Err(e) => {
                        failures += 1;
                        eprintln!(
                            "[fail] {} (size: {},{}) via external '{}': {} in {:.2?}",
                            path.display(),
                            game.num_nodes(),
                            game.num_edges(),
                            external_command,
                            e,
                            duration
                        );
                    }
                }

                // Clean up the temporary output file
                if let Err(e) = std::fs::remove_file(&output_file) {
                    eprintln!(
                        "Warning: Failed to remove temporary file '{}': {}",
                        output_file.display(),
                        e
                    );
                }
            }
        }
    } else {
        let count = random_count.unwrap_or(100);
        let nodes = random_nodes.unwrap_or(100);
        let max_edges = max_edges.unwrap_or(4);
        let max_prio = max_prio.unwrap_or(nodes);

        for i in 0..count {
            let game = ParityGameBuilder::new()
                .random_game(nodes, max_edges, max_prio, seed)
                .build();
            for algorithm in &algorithms {
                let start_time = std::time::Instant::now();
                let result = solve_game(&game, algorithm);
                let duration = start_time.elapsed();
                combined_times
                    .entry(algorithm.clone())
                    .and_modify(|d| *d += duration)
                    .or_insert(duration);
                let (w0, w1, strat0, strat1) = match result {
                    Ok(res) => res,
                    Err(e) => {
                        failures += 1;
                        eprintln!(
                            "[fail] random game #{} (size: {},{}) via {}: Error running algorithm: {} in {:.2?}",
                            i + 1,
                            game.num_nodes(),
                            game.num_edges(),
                            algorithm,
                            e,
                            duration
                        );
                        continue;
                    }
                };
                match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
                    Ok(()) => {
                        println!(
                            "[ok] random game #{} (size: {},{}) via {} in {:.2?}",
                            i + 1,
                            game.num_nodes(),
                            game.num_edges(),
                            algorithm,
                            duration
                        );
                    }
                    Err(e) => {
                        failures += 1;
                        eprintln!(
                            "[fail] random game #{} (size: {},{}) via {}: {} in {:.2?}",
                            i + 1,
                            game.num_nodes(),
                            game.num_edges(),
                            algorithm,
                            e,
                            duration
                        );
                    }
                }
            }

            let input_file =
                std::env::temp_dir().join(format!("parity-forge-random-game-{}.pg", i + 1));
            if !external_commands.is_empty() {
                std::fs::write(&input_file, unparse_pg(&game)).unwrap_or_else(|e| {
                    eprintln!(
                        "Error writing temporary game file '{}': {}",
                        input_file.display(),
                        e
                    );
                    std::process::exit(1);
                });
            }

            for external_command in &external_commands {
                let output_file = std::env::temp_dir()
                    .join(format!("parity-forge-random-test-{}.paritysol", i + 1));
                let start_time = std::time::Instant::now();
                let result = run_external_solver(external_command, &input_file, &output_file);
                let duration = start_time.elapsed();
                combined_times
                    .entry(external_command.clone())
                    .and_modify(|d| *d += duration)
                    .or_insert(duration);
                match result {
                    Ok((w0, w1, strat0, strat1)) => {
                        match verify_solution(&game, &w0, &w1, &strat0, &strat1) {
                            Ok(()) => {
                                println!(
                                    "[ok] random game #{} (size: {},{}) via external '{}' in {:.2?}",
                                    i + 1,
                                    game.num_nodes(),
                                    game.num_edges(),
                                    external_command,
                                    duration
                                );
                            }
                            Err(e) => {
                                failures += 1;
                                eprintln!(
                                    "[fail] random game #{} (size: {},{}) via external '{}': {} in {:.2?}",
                                    i + 1,
                                    game.num_nodes(),
                                    game.num_edges(),
                                    external_command,
                                    e,
                                    duration
                                );
                            }
                        }
                    }
                    Err(e) => {
                        failures += 1;
                        eprintln!(
                            "[fail] random game #{} (size: {},{}) via external '{}': {} in {:.2?}",
                            i + 1,
                            game.num_nodes(),
                            game.num_edges(),
                            external_command,
                            e,
                            duration
                        );
                    }
                }

                // Clean up the temporary output file
                if let Err(e) = std::fs::remove_file(&output_file) {
                    eprintln!(
                        "Warning: Failed to remove temporary file '{}': {}",
                        output_file.display(),
                        e
                    );
                }
            }

            // Clean up the temporary input file
            if !external_commands.is_empty()
                && let Err(e) = std::fs::remove_file(&input_file)
            {
                eprintln!(
                    "Warning: Failed to remove temporary file '{}': {}",
                    input_file.display(),
                    e
                );
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
        Ok(()) => println!(
            "Solution in '{}' is valid for game '{}'",
            solution_file, game_file
        ),
        Err(e) => {
            eprintln!(
                "Solution in '{}' is invalid for game '{}': {}",
                solution_file, game_file, e
            );
            std::process::exit(1);
        }
    }
}

fn run_algorithm<Algo>(input_file: &str, output_file: &str, algorithm: Algo, alg_name: &str)
where
    Algo: Fn(
        &ParityGame,
    ) -> Result<
        (
            Vec<usize>,
            Vec<usize>,
            Vec<Option<usize>>,
            Vec<Option<usize>>,
        ),
        String,
    >,
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
        let output = strat_to_sol(
            &game,
            &strategy_0,
            &strategy_1,
            &winning_region0,
            &winning_region1,
        );

        std::fs::write(output_file, output).unwrap_or_else(|e| {
            eprintln!("Error writing output file '{}': {}", output_file, e);
            std::process::exit(1);
        });
        println!("Solution written to '{}'", output_file);
    }
}

fn zielonka(input: &str, output_file: &str) {
    run_algorithm(
        input,
        output_file,
        run_zielonka,
        "Zielonka's Recursive Algorithm",
    );
}

fn unoptimized_zielonka(input: &str, output_file: &str) {
    run_algorithm(
        input,
        output_file,
        run_unoptimized_zielonka,
        "Unoptimized Zielonka's Recursive Algorithm",
    );
}

fn fpi(input: &str, output_file: &str) {
    run_algorithm(
        input,
        output_file,
        run_fpi,
        "Fixed-Point Iteration Algorithm",
    );
}

fn tl(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_tl, "Tangle Learning Algorithm");
}

fn spm(input: &str, output_file: &str) {
    run_algorithm(
        input,
        output_file,
        run_spm,
        "Small Progress Measures Algorithm",
    );
}

fn si(input: &str, output_file: &str) {
    run_algorithm(input, output_file, run_si, "Strategy Improvement Algorithm");
}

fn solve_game(
    game: &ParityGame,
    algorithm: &str,
) -> Result<
    (
        Vec<usize>,
        Vec<usize>,
        Vec<Option<usize>>,
        Vec<Option<usize>>,
    ),
    String,
> {
    match algorithm {
        "default" | "zlk" => run_zielonka(game),
        "uzlk" => run_unoptimized_zielonka(game),
        "fpi" => run_fpi(game),
        "tl" => run_tl(game),
        "spm" => run_spm(game),
        "si" => run_si(game),
        _ => {
            eprintln!("Algorithm '{}' not implemented yet.", algorithm);
            std::process::exit(2);
        }
    }
}

fn collect_pg_inputs(path: &Path) -> Result<Vec<PathBuf>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Error reading '{}': {}", path.display(), e))?;
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

    Err(format!(
        "'{}' is neither a file nor a directory",
        path.display()
    ))
}

fn collect_pg_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir)
        .map_err(|e| format!("Error reading directory '{}': {}", dir.display(), e))?
    {
        let entry = entry.map_err(|e| {
            format!(
                "Error reading directory entry in '{}': {}",
                dir.display(),
                e
            )
        })?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|e| format!("Error reading metadata for '{}': {}", path.display(), e))?;

        if metadata.is_dir() {
            collect_pg_files_recursive(&path, files)?;
        } else if metadata.is_file() && is_pg_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_pg_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pg"))
}
