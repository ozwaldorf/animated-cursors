use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "Usage: dot-cursors --output PATH [--preview PATH]";

fn main() -> ExitCode {
    let (mut output, mut preview) = (None, None);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--output", Some(path)) => output = Some(PathBuf::from(path)),
            ("--preview", Some(path)) => preview = Some(PathBuf::from(path)),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(output) = output else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    if output.exists() {
        eprintln!("Output already exists; choose a fresh directory");
        return ExitCode::FAILURE;
    }
    match dot_cursors::theme::generate(&output, preview.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
