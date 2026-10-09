//! Shared infrastructure for animated cursor themes: shape names, motion
//! helpers, painting utilities and the theme writer.

pub mod check;
pub mod cli;
pub mod motion;
pub mod paint;
pub mod shape;
pub mod theme;
pub mod xcursor;

pub use shape::Shape;
pub use theme::{Frame, Theme};
