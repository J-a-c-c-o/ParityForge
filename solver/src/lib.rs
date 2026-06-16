pub mod parity_game;
pub mod pg_parser;
pub mod solvers;
pub mod verifier;

pub use parity_game::{ParityGame, ParityGameBuilder};
use std::path::Path;
use std::fs;

// Struct to hold the solution data
pub struct Solution {
    pub w0: Vec<usize>,
    pub w1: Vec<usize>,
    pub strat0: Vec<Option<usize>>,
    pub strat1: Vec<Option<usize>>,
}

/// Enum for type-safe algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Zielonka,
    UnoptimizedZielonka,
    Fpi,
    Fpj,
    Tl,
    Ptl,
    Si,
    Spm,
}

impl std::str::FromStr for Algorithm {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zlk" => Ok(Algorithm::Zielonka),
            "uzlk" => Ok(Algorithm::UnoptimizedZielonka),
            "fpi" => Ok(Algorithm::Fpi),
            "fpj" => Ok(Algorithm::Fpj),
            "tl" => Ok(Algorithm::Tl),
            "ptl" => Ok(Algorithm::Ptl),
            "si" => Ok(Algorithm::Si),
            "spm" => Ok(Algorithm::Spm),
            _ => Err(format!("Unknown algorithm: {}", s)),
        }
    }
}

pub fn load_pg(path: &Path) -> Result<ParityGame, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    pg_parser::parse_pg(&text).map_err(|e| e.to_string())
}

pub fn save_solution(path: &Path, game: &ParityGame, sol: &Solution) -> Result<(), String> {
    let output = pg_parser::strat_to_sol(
        game, &sol.strat0, &sol.strat1, &sol.w0, &sol.w1
    );
    fs::write(path, output).map_err(|e| e.to_string())
}

pub fn load_solution(path: &Path) -> Result<Solution, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (w0, w1, strat0, strat1) = pg_parser::sol_to_strat(&text).map_err(|e| e.to_string())?;
    Ok(Solution { w0, w1, strat0, strat1 })
}

pub fn solve(game: &ParityGame, algo: Algorithm) -> Result<Solution, String> {
    let (w0, w1, strat0, strat1) = match algo {
        Algorithm::Zielonka => solvers::run_zielonka(game)?,
        Algorithm::UnoptimizedZielonka => solvers::run_unoptimized_zielonka(game)?,
        Algorithm::Fpi => solvers::run_fpi(game)?,
        Algorithm::Fpj => solvers::run_fpj(game)?,
        Algorithm::Tl => solvers::run_tl(game)?,
        Algorithm::Ptl => solvers::run_ptl(game)?,
        Algorithm::Si => solvers::run_si(game)?,
        Algorithm::Spm => solvers::run_spm(game)?,
    };
    Ok(Solution { w0, w1, strat0, strat1 })
}

pub fn verify(game: &ParityGame, sol: &Solution) -> Result<(), String> {
    verifier::verify_solution(game, &sol.w0, &sol.w1, &sol.strat0, &sol.strat1)
}

pub fn generate_random_pg(nodes: usize, max_edges: usize, max_prio: usize, seed: Option<u64>) -> ParityGame {
    ParityGameBuilder::new()
        .random_game(nodes, max_edges, max_prio, seed)
        .build()
}