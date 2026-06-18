# ParityForge

## About

This repository provides a high-performance, command-line toolchain written in Rust for parsing, solving, verifying, and benchmarking parity games.

The project is structured as a Cargo workspace and includes implementations of several well-known parity game solving algorithms:

- **Zielonka's Recursive Algorithm**
- W. Zielonka, Infinite games on finitely coloured graphs with applications to automata on infinite trees. Theoret. Comput. Sci. 200 (1998) 135–183.

- **Tangle Learning**
- T. van Dijk, Attracting tangles to solve parity games, in: CAV (2), Springer, 2018, pp. 198–215.

- **Small Progress Measures**
- M. Jurdziński (2000): Small Progress Measures for Solving Parity Games. In: STACS’00, Lecture Notes in Computer Science 1770, Springer, pp. 290–301.

- **Strategy Improvement**
- Fearnley, J.: Efficient parallel strategy improvement for parity games. In: Majumdar, R., Kunčak, V. (eds.) CAV 2017. LNCS, vol. 10427, pp. 137–154. Springer, Cham (2017).

- **Fixed-Point Iteration with Justifications**
- Ruben Lapauw, Maurice Bruynooghe & Marc Denecker (2020): Improving Parity Game Solvers with Justifications. In: VMCAI, Lecture Notes in Computer Science 11990, Springer, pp. 449–470.

- **Fixed-Point Iteration with Freezing Sets**
- van Dijk, T., Rubbens, B.: Simple fixpoint iteration to solve parity games. In: GandALF. EPTCS, vol. 305, pp. 123–139 (2019).

---

## Project Structure

The project is organized as a Rust workspace containing three main crates:

```text
.
├── Cargo.toml               # Workspace root configuration
├── cli/                     # The primary command-line interface
│   ├── Cargo.toml
│   └── src/main.rs
├── solver/                  # The core library containing all logic
│   ├── Cargo.toml
│   ├── benches/             # Criterion statistical benchmarks
│   │   └── benchmark.rs
│   └── src/
│       ├── lib.rs
│       ├── parity_game.rs
│       ├── pg_parser.rs
│       ├── solvers          # Module containing all solver implementations
│       │   ├── mod.rs
│       │   ├── fpi.rs       # Fixed-Point Iteration with Freezing Sets
│       │   ├── fpj.rs       # Fixed-Point Iteration with Justifications
│       │   ├── ptl.rs       # Parallel Tangle Learning
│       │   ├── si.rs        # Strategy Improvement
│       │   ├── spm.rs       # Small Progress Measures
│       │   ├── tl.rs        # Tangle Learning
│       │   ├── pzlk.rs      # Parallel Zielonka
│       │   └── zlk.rs       # Zielonka
│       └── verifier.rs
├── test_solvers/            # A dedicated bulk-testing and CSV generation client
│   ├── Cargo.toml
│   └── src/main.rs
└── experiments/             # Contains csv files and ipynb notebook for visualizing and analyzing
    └── report.ipynb         # Jupyter notebook for visualizing and analyzing the results of the experiments
```

---

## Installation Instructions

### 1. Prerequisites

Ensure you have the Rust toolchain installed. If you do not have Rust installed, you can install it via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```

### 2. Clone the Repository

```bash
git clone https://github.com/J-a-c-c-o/ParityForge.git
cd ParityForge

```

### 3. Build the Project

Because this is a compute-heavy application, it is highly recommended to build and run the tools in `--release` mode for maximum performance.

```bash
cargo build --release

```

---

## Usage Examples

Because this is a Rust workspace, you can easily run the different binaries from the root directory using `cargo run -p <package_name>`.

### The Main CLI (`cli`)

The standard CLI is used for solving individual games and saving the results.

**Solve a game and save the solution:**

```bash
cargo run --release -p cli -- solve examples/oink/abcg_arbiter.tlsf.ehoa.pg output/solution.sol -a zlk

```

**Verify an existing solution:**

```bash
cargo run --release -p cli -- verify examples/oink/abcg_arbiter.tlsf.ehoa.pg  output/solution.sol

```

**List all available algorithms:**

```bash
cargo run --release -p cli -- list-algorithms
```

**Help and usage information:**

```bash
cargo run --release -p cli -- help
```

### The Bulk Testing Client (`test_solvers`)

This tool is designed to run multiple algorithms against large directories of games (or randomly generated games) to validate correctness and export performance metrics.

**Test specific algorithms on a directory of `.pg` files and export to CSV:**

```bash
cargo run --release -p test-solvers -- test -i examples/oink/ -a zlk -a tl -a fpi --csv results.csv

```

**Generate and test 100 random games (1000 nodes each):**

```bash
cargo run --release -p test-solvers -- test --count 100 --size 1000 -a fpi,fpj

```

### Statistical Benchmarking (`solver/benches`)

To run rigorous performance and scaling benchmarks using `criterion`, use the `cargo bench` command.

```bash
# Run with default sizes (50, 100)
cargo bench -p solver

# Run with specific custom sizes
BENCH_SIZES=200,500,1000 cargo bench -p solver

```

_Note: You can filter the benchmark execution by algorithm name by appending it to the command (e.g., `cargo bench -p solver -- Zielonka`)._

---

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
