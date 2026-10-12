//! Theme colors by role, loaded from `role = #rrggbb` files.

use std::ops::Index;

use crate::color::{Color, hex};
use crate::theme::Theme;

pub struct Palette {
    pub name: String,
    colors: Vec<(String, Color)>,
}

impl Palette {
    /// Parses `role = #rrggbb` lines, skipping blanks and `#` comments. Every
    /// role in `roles` must appear exactly once, and nothing else.
    pub fn parse(name: &str, text: &str, roles: &[&str]) -> Result<Palette, String> {
        let mut colors: Vec<(String, Color)> = Vec::new();
        for (number, line) in text.lines().enumerate().map(|(i, line)| (i + 1, line.trim())) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let entry = line.split_once('=').and_then(|(role, value)| {
                let digits = value.trim().strip_prefix('#')?;
                let valid = digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit());
                Some((role.trim(), u32::from_str_radix(digits, 16).ok().filter(|_| valid)?))
            });
            let Some((role, rgb)) = entry else {
                return Err(format!("{name}:{number}: expected `role = #rrggbb`"));
            };
            if !roles.contains(&role) {
                return Err(format!("{name}:{number}: unknown role `{role}`"));
            }
            if colors.iter().any(|(known, _)| known == role) {
                return Err(format!("{name}:{number}: `{role}` is set twice"));
            }
            colors.push((role.to_owned(), hex(rgb)));
        }
        match roles.iter().find(|role| !colors.iter().any(|(known, _)| known == *role)) {
            Some(role) => Err(format!("{name}: missing role `{role}`")),
            None => Ok(Palette { name: name.to_owned(), colors }),
        }
    }

    /// A bundled palette by name, or a palette file by path.
    pub fn load<T: Theme>(choice: &str) -> Result<Palette, String> {
        if let Some((name, text)) = T::PALETTES.iter().find(|(name, _)| *name == choice) {
            return Palette::parse(name, text, T::ROLES);
        }
        let path = std::path::Path::new(choice);
        let text = std::fs::read_to_string(path).map_err(|error| format!("{choice}: {error}"))?;
        let name = path.file_stem().map_or(choice.into(), |stem| stem.to_string_lossy());
        Palette::parse(&name, &text, T::ROLES)
    }
}

impl Index<&str> for Palette {
    type Output = Color;

    fn index(&self, role: &str) -> &Color {
        let found = self.colors.iter().find(|(known, _)| known == role);
        &found.unwrap_or_else(|| panic!("no color for `{role}`")).1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rejects() {
        let palette = Palette::parse("test", "# comment\n\nink = #ff0000\nrim=#00FF00\n", &["ink", "rim"]).unwrap();
        assert_eq!(palette["ink"], hex(0xff0000));
        assert_eq!(palette["rim"], hex(0x00ff00));
        for (text, error) in [
            ("ink = #ff0000", "missing role `rim`"),
            ("ink = #ff0000\nrim = #00ff00\nfoo = #000000", "unknown role `foo`"),
            ("ink = #ff0000\nink = #00ff00", "set twice"),
            ("ink = ff0000", "expected"),
            ("ink = #+f0000", "expected"),
        ] {
            let result = Palette::parse("test", text, &["ink", "rim"]);
            assert!(result.as_ref().is_err_and(|e| e.contains(error)), "{text}");
        }
    }
}
