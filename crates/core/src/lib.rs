//! Shared infrastructure for animated cursor themes: shape names, motion
//! helpers, colors, painting utilities and the theme writer.

pub mod check;
pub mod cli;
pub mod color;
pub mod motion;
pub mod paint;
pub mod palette;
pub mod shape;
pub mod theme;
pub mod xcursor;

pub use palette::Palette;
pub use shape::Shape;
pub use theme::{Frame, Theme};
