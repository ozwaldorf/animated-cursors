//! Command line entry point shared by every theme binary.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::palette::Palette;
use crate::theme::{SIZES, Theme, generate, sheet};

pub fn run<T: Theme>() -> ExitCode {
    let names: Vec<&str> = T::PALETTES.iter().map(|(name, _)| *name).collect();
    let usage = format!(
        "Usage: {} (--output PATH [--preview PATH] [--sizes 48,72] | --sheet PATH) [--palette {}|FILE]",
        T::ID,
        names.join("|")
    );
    let (mut output, mut preview, mut still, mut sizes, mut palette) =
        (None, None, None, SIZES.to_vec(), names[0].to_owned());
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--output", Some(path)) => output = Some(PathBuf::from(path)),
            ("--preview", Some(path)) => preview = Some(PathBuf::from(path)),
            ("--sheet", Some(path)) => still = Some(PathBuf::from(path)),
            ("--palette", Some(choice)) => palette = choice,
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
    if output.is_some() == still.is_some() {
        eprintln!("{usage}");
        return ExitCode::FAILURE;
    }
    if output.as_ref().is_some_and(|output| output.exists()) {
        eprintln!("Output already exists; choose a fresh directory");
        return ExitCode::FAILURE;
    }
    let theme = match Palette::load::<T>(&palette) {
        Ok(palette) => T::new(palette),
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let result = match (output, still) {
        (Some(output), _) => generate(&theme, &output, preview.as_deref(), &sizes),
        (_, Some(path)) => sheet(&theme, &path),
        (None, None) => unreachable!(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
