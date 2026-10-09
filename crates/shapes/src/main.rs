use std::process::ExitCode;

fn main() -> ExitCode {
    cursor_core::cli::run(&shapes_cursors::Shapes)
}
