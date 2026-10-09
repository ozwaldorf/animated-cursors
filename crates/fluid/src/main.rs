use std::process::ExitCode;

fn main() -> ExitCode {
    cursor_core::cli::run(&fluid_cursors::Fluid)
}
