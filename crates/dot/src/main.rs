use std::process::ExitCode;

fn main() -> ExitCode {
    cursor_core::cli::run(&dot_cursors::Dot)
}
