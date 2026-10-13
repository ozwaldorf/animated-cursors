use std::process::ExitCode;

fn main() -> ExitCode {
    cursor_core::cli::run::<hand_cursors::Hand>()
}
