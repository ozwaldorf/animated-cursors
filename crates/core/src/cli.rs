//! Command line entry point shared by every theme binary.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::theme::{Theme, generate};

pub fn run<T: Theme>(theme: &T) -> ExitCode {
    let usage = format!("Usage: {} --output PATH [--preview PATH]", T::ID);
    let (mut output, mut preview) = (None, None);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--output", Some(path)) => output = Some(PathBuf::from(path)),
            ("--preview", Some(path)) => preview = Some(PathBuf::from(path)),
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
    match generate(theme, &output, preview.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
