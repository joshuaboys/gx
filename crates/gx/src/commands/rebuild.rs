//! `gx rebuild` — rescan the project dir and rebuild the index in place.
//! Ported from `src/commands/rebuild.ts`.

use std::path::Path;

use crate::config::effective_project_dir;
use crate::errors::GxResult;
use crate::index_store::ProjectIndex;
use crate::types::Config;

pub fn rebuild(config: &Config, index_path: &Path) -> GxResult<()> {
    let project_dir = effective_project_dir(config)?;
    let mut idx = ProjectIndex::load(index_path);
    idx.scoped_rebuild(&project_dir)?;
    idx.save(index_path)?;
    let count = idx.list().len();
    eprintln!("Indexed {count} projects");
    Ok(())
}
