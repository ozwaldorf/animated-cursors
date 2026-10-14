//! Command line entry point shared by every theme binary.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::palette::Palette;
use crate::preview::previews;
use crate::theme::{SIZES, Theme, generate};

pub fn run<T: Theme>() -> ExitCode {
    let names: Vec<&str> = T::PALETTES.iter().map(|(name, _)| *name).collect();
    let usage = format!(
        "Usage: {} [--output PATH [--sizes 48,72]] [--preview DIR] [--palette {}|FILE]",
        T::ID,
        names.join("|")
    );
    let (mut output, mut preview, mut sizes, mut palette) = (None, None, SIZES.to_vec(), names[0].to_owned());
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--output", Some(path)) => output = Some(PathBuf::from(path)),
            ("--preview", Some(path)) => preview = Some(PathBuf::from(path)),
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
    if output.is_none() && preview.is_none() {
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
    let result = output
        .map_or(Ok(()), |output| generate(&theme, &output, &sizes))
        .and_then(|()| preview.map_or(Ok(()), |directory| previews(&theme, &directory)));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
