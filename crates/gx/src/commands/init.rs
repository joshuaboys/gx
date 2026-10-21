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

    // 2. Preflight every output before writing any files.
    let claude_dir = dir.join(".claude");
    let claude_md = claude_dir.join("CLAUDE.md");
    let commands_dir = claude_dir.join("commands");

    if !force {
        for relative in [
            ".claude/CLAUDE.md",
            ".claude/commands/plan.md",
            ".claude/commands/review.md",
        ] {
            // symlink_metadata also catches dangling links.
            match std::fs::symlink_metadata(dir.join(relative)) {
                Ok(_) => {
                    return Err(GxError::Other(format!(
                        "{relative} already exists. Use --force to overwrite."
                    )))
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(GxError::Other(format!("inspect {relative}: {e}"))),
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_every_existing_output_without_force() {
        for name in ["CLAUDE.md", "commands/plan.md", "commands/review.md"] {
            let dir = tempfile::tempdir().unwrap();
            let existing = dir.path().join(".claude").join(name);
            std::fs::create_dir_all(existing.parent().unwrap()).unwrap();
            std::fs::write(&existing, "user customization").unwrap();
            assert!(init_agent(dir.path(), Some("generic"), false).is_err());
            assert_eq!(
                std::fs::read_to_string(&existing).unwrap(),
                "user customization"
            );
            if name != "CLAUDE.md" {
                assert!(!dir.path().join(".claude/CLAUDE.md").exists());
            }
            init_agent(dir.path(), Some("generic"), true).unwrap();
            assert_ne!(
                std::fs::read_to_string(existing).unwrap(),
                "user customization"
            );
        }
    }
}
