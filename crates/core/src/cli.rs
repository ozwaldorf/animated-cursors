//! Command line entry point shared by every theme binary.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::theme::{SIZES, Theme, generate};

pub fn run<T: Theme>(theme: &T) -> ExitCode {
    let usage = format!("Usage: {} --output PATH [--preview PATH] [--sizes 48,72]", T::ID);
    let (mut output, mut preview, mut sizes) = (None, None, SIZES.to_vec());
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--output", Some(path)) => output = Some(PathBuf::from(path)),
            ("--preview", Some(path)) => preview = Some(PathBuf::from(path)),
            ("--sizes", Some(list)) => {
                match list.split(',').map(|size| size.trim().parse()).collect::<Result<Vec<u32>, _>>() {
                    Ok(parsed) if parsed.iter().all(|&size| size > 0) && !parsed.is_empty() => sizes = parsed,
                    _ => {
                        eprintln!("Sizes are positive pixel sizes separated by commas");
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ => {
                eprintln!("{usage}");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(output) = output else {
        eprintln!("{usage}");
        return ExitCode::FAILURE;
    };
    if output.exists() {
        eprintln!("Output already exists; choose a fresh directory");
        return ExitCode::FAILURE;
    }
    match generate(theme, &output, preview.as_deref(), &sizes) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
