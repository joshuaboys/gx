//! User-facing command implementations. Each module mirrors the matching
//! `src/commands/*.ts` file under strict behaviour parity.

pub mod clone;
pub mod config_cmd;
pub mod doctor;
pub mod index_repos;
pub mod init;
pub mod ls;
pub mod open;
pub mod rebuild;
pub mod recent;
pub mod resolve;
pub mod resume;
pub mod shell_init;
