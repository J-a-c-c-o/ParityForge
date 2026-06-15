use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// This imports the pure API from your library crate
use solver::{generate_random_pg, load_pg, solve, verify, Algorithm};

/// ParityForge Tester
#[derive(Parser)]
#[command(
    name = "test-solvers",
    about = "Bulk testing client for parity game algorithms"
)]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Test {
        /// Input file or directory (if omitted, runs random games)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Algorithms to test (default: all)
        #[arg(long, short = 'a')]
        algorithm: Vec<String>,

        /// Number of random games to test
        #[arg(short, long, default_value_t = 100)]
        count: usize,

        /// Number of nodes for random games
        #[arg(short, long, default_value_t = 100)]
        size: usize,

        /// Maximum edges per node for random games
        #[arg(short, long, default_value_t = 4)]
        maxe: usize,

        /// Maximum priority for random games
        #[arg(short, long, default_value_t = 50)]
        maxp: usize,

        /// Optional seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Test { input, algorithm, count, size, maxe, maxp, seed } => {
            let mut failures = 0;
            let mut combined_times: HashMap<&str, Duration> = HashMap::new();

            let algorithms = if algorithm.is_empty() {
                vec![
                    ("Zielonka", Algorithm::Zielonka),
                    ("Unoptimized Zielonka", Algorithm::UnoptimizedZielonka),
                    ("FPI", Algorithm::Fpi),
                    ("FPJ", Algorithm::Fpj),
                    ("TL", Algorithm::Tl),
                    ("PTL", Algorithm::Ptl),
                    ("SI", Algorithm::Si),
                    ("SPM", Algorithm::Spm),
                ]
            } else {
                algorithm.iter().map(|name| {
                    let algo = name.parse::<Algorithm>().unwrap_or_else(|e| {
                        eprintln!("Error parsing algorithm '{}': {}", name, e);
                        std::process::exit(1);
                    });
                    (name.as_str(), algo)
                }).collect()
            };

            if let Some(path) = input {
                let input_paths = collect_pg_inputs(&path).unwrap_or_else(|e| {
                    eprintln!("Failed to read input path: {}", e);
                    std::process::exit(1);
                });

                if input_paths.is_empty() {
                    eprintln!("No .pg files found under '{}'", path.display());
                    std::process::exit(1);
                }

                for file_path in input_paths {
                    let game = match load_pg(&file_path) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("[ERROR] Failed to load '{}': {}", file_path.display(), e);
                            continue;
                        }
                    };

                    for (name, algo) in &algorithms {
                        let start = Instant::now();
                        match solve(&game, algo.clone()) {
                            Ok(sol) => {
                                let duration = start.elapsed();
                                *combined_times.entry(name).or_insert(Duration::ZERO) += duration;

                                if let Err(e) = verify(&game, &sol) {
                                    eprintln!("[FAIL] {} via {}: {}", file_path.display(), name, e);
                                    failures += 1;
                                } else {
                                    println!("[OK] {} via {} in {:?}", file_path.display(), name, duration);
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] {} via {}: {}", file_path.display(), name, e);
                                failures += 1;
                            }
                        }
                    }
                }
            } else {
                println!(
                    "Generating {} random games (size: {}, max_edges: {}, max_prio: {})...",
                    count, size, maxe, maxp
                );

                for i in 0..count {
                    let current_seed = seed.map(|s| s + i as u64);
                    let game = generate_random_pg(size, maxe,   maxp, current_seed);

                    for (name, algo) in &algorithms {
                        let start = Instant::now();
                        match solve(&game, algo.clone()) {
                            Ok(sol) => {
                                let duration = start.elapsed();
                                *combined_times.entry(name).or_insert(Duration::ZERO) += duration;

                                if let Err(e) = verify(&game, &sol) {
                                    eprintln!("[FAIL] Random Game #{} via {}: {}", i + 1, name, e);
                                    failures += 1;
                                } else {
                                    println!("[OK] Random Game #{} via {} in {:?}", i + 1, name, duration);
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Random Game #{} via {}: {}", i + 1, name, e);
                                failures += 1;
                            }
                        }
                    }
                }
            }

            // Print summary statistics
            println!("\n--- Test Summary ---");
            println!("Combined execution times:");
            for (name, duration) in combined_times {
                println!("  {}: {:?}", name, duration);
            }

            if failures == 0 {
                println!("\nSUCCESS: All tests passed!");
            } else {
                eprintln!("\nFAILURE: {} test run(s) failed.", failures);
                std::process::exit(1);
            }
        }
    }
}

// Helper function to collect files from a given path (file or directory)
fn collect_pg_inputs(path: &Path) -> Result<Vec<PathBuf>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Error reading '{}': {}", path.display(), e))?;
    let mut files = Vec::new();

    if metadata.is_file() {
        files.push(path.to_path_buf());
        return Ok(files);
    }

    if metadata.is_dir() {
        collect_files_recursive(path, &mut files)?;
        files.sort();
        return Ok(files);
    }

    Err(format!(
        "'{}' is neither a file nor a directory",
        path.display()
    ))
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("Error reading dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Error reading entry: {}", e))?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|e| format!("Metadata error: {}", e))?;

        if metadata.is_dir() {
            collect_files_recursive(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}