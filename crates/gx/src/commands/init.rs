//! `gx init [--type <type>] [--force]` — scaffold `.claude/` agent config.
//! Ported from `src/commands/init.ts`.

use std::path::Path;

use crate::detect::{detect_project_type, ProjectType};
use crate::errors::{GxError, GxResult};
use crate::templates::{get_claude_md, get_plan_command, get_review_command};

/// Mirrors the TS `VALID_TYPES` set and its iteration order, used for the
/// "Unknown project type" error message.
const VALID_TYPES: &str = "typescript-bun, typescript-node, rust, go, python, generic";

pub fn init_agent(dir: &Path, type_opt: Option<&str>, force: bool) -> GxResult<()> {
    // 1. Determine project type.
    let project_type = match type_opt {
        Some(t) => ProjectType::from_str_opt(t).ok_or_else(|| {
            GxError::Other(format!(
                "Unknown project type '{t}'. Valid types: {VALID_TYPES}"
            ))
        })?,
        None => detect_project_type(dir),
    };

    // 2. Refuse to overwrite an existing CLAUDE.md unless forced.
    let claude_dir = dir.join(".claude");
    let claude_md = claude_dir.join("CLAUDE.md");
    let commands_dir = claude_dir.join("commands");

    if !force && claude_md.exists() {
        return Err(GxError::Other(
            ".claude/CLAUDE.md already exists. Use --force to overwrite.".to_string(),
        ));
    }

    // 3. Create directories.
    std::fs::create_dir_all(&commands_dir)
        .map_err(|e| GxError::Other(format!("create {}: {e}", commands_dir.display())))?;

    // 4. Write files.
    write_file(&claude_md, get_claude_md(project_type))?;
    write_file(&commands_dir.join("plan.md"), get_plan_command())?;
    write_file(&commands_dir.join("review.md"), get_review_command())?;

    // 5. Report.
    eprintln!("Scaffolded .claude/ for {} project:", project_type.as_str());
    eprintln!("  .claude/CLAUDE.md");
    eprintln!("  .claude/commands/plan.md");
    eprintln!("  .claude/commands/review.md");
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> GxResult<()> {
    std::fs::write(path, contents)
        .map_err(|e| GxError::Other(format!("write {}: {e}", path.display())))
}
