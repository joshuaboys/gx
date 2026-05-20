//! Library surface for the gx Rust port.
//!
//! All pure-logic modules — URL parsing, path mapping, fuzzy matching,
//! config (de)serialisation, project-type detection, scaffold templates,
//! relative time, and the typed error model — live here so they can be
//! exercised by unit tests and snapshot tests without spawning the binary.

pub mod config;
pub mod detect;
pub mod errors;
pub mod fuzzy;
pub mod path;
pub mod templates;
pub mod time;
pub mod types;
pub mod url;
