use std::{path::PathBuf, process::Command};
use crate::pg_parser::sol_to_strat;

pub fn run_external_solver(
    command: &str,
    input_file: &PathBuf,
    output_file: &PathBuf,
) -> Result<
    (
        Vec<usize>,
        Vec<usize>,
        Vec<Option<usize>>,
        Vec<Option<usize>>,
    ),
    String,
> {
    let command = command.replace("%I", &input_file.to_string_lossy()).replace("%O", &output_file.to_string_lossy());

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)        
        ));
    }

    let output_content = std::fs::read_to_string(output_file)
        .map_err(|e| format!("Failed to read output file: {}", e))?;

    Ok(sol_to_strat(&output_content)?)
}

